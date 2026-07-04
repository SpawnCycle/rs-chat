use std::{collections::VecDeque, time::Instant};

use chat_lib::{
    prelude::*,
    types::{Message as ChatMessage, Sync},
    ws_connection::{Message, WsConnection},
};
use futures::{SinkExt, StreamExt};
use rustrict::Context;
use tokio::sync::broadcast::{self, error::RecvError};
use uuid::Uuid;

use crate::ws::{MsgBroadcastReceiver, MsgBroadcastSender, Room, consts::TIMEOUT_WINDOW};
use crate::{
    config::CONTEXT_OPTS,
    ws::consts::{MESSAGE_LIMIT, TIMEOUT_DURATION},
};

pub type WsResult<T = ()> = Result<T, anyhow::Error>;

pub struct WsHandler<'a, F>
where
    F: Future<Output = ()> + Clone,
{
    stream: WsConnection,
    room: Sync<Room>,
    message_counter: VecDeque<Instant>,
    ctx: Context,
    id: Uuid,
    rx: MsgBroadcastReceiver,
    tx: MsgBroadcastSender,
    sd: &'a mut F,
    stream_open: bool,
    in_room: bool,
}

impl<'a, F> WsHandler<'a, F>
where
    F: Future<Output = ()> + Clone,
{
    pub const fn new(
        stream: WsConnection,
        ctx: Context,
        id: Uuid,
        rx: MsgBroadcastReceiver,
        tx: MsgBroadcastSender,
        room: Sync<Room>,
        sd: &'a mut F,
    ) -> Self {
        Self {
            stream,
            ctx,
            room,
            id,
            rx,
            tx,
            sd,
            message_counter: VecDeque::new(),
            stream_open: true,
            in_room: true,
        }
    }

    pub async fn ws_step(&mut self) -> WsResult<bool> {
        if !self.in_room {
            return Ok(true);
        }
        tokio::select! {
            Some(res) = self.stream.next() => {
                self.handle_stream(res).await
            }
            res = self.rx.recv() => self.handle_rx(res).await,
            () = self.sd.clone() => {
                self.close_logged().await;
                Ok(true)
            }
            else => {
                self.close_logged().await;
                Ok(true)
            }
        }
    }

    // This doesn't feel right, but at least it processes all of the messages,
    // which may or may not be the right thing to do
    pub async fn cleanup(&mut self) -> WsResult<()> {
        while let Some(res) = self.stream.next().await {
            self.handle_stream(res).await?;
        }

        Ok(())
    }

    async fn handle_stream(&mut self, res: Result<Message, anyhow::Error>) -> WsResult<bool> {
        match res {
            Err(err) => {
                if self.stream_open {
                    self.close_socket().await?;
                }
                Err(err)
            }
            Ok(msg) => {
                self.count_message();
                if !self.can_send_message() {
                    self.send_timeout_message().await?;
                    return Ok(true);
                }
                match msg {
                    Message::Text(txt) => self.handle_text(&txt).await,
                    Message::Close(_) => {
                        if self.stream_open {
                            self.exit_room().await;
                        }
                        Ok(true)
                    }
                    _ => {
                        if self.stream_open {
                            self.close_logged().await;
                        }
                        Ok(true)
                    }
                }
            }
        }
    }

    async fn handle_text(&mut self, txt: &str) -> WsResult<bool> {
        if let Ok(msg) = serde_json::from_str::<ClientMessage>(txt) {
            log::debug!("Processing message: {msg:?}");

            match msg {
                ClientMessage::SendMessage(msg) => {
                    self.send_msg(&msg).await?;
                }
                ClientMessage::ChangeUserName(name) => {
                    let mut room = self.room.lock().await;
                    if name.chars().count() > MAX_NAME_LENGTH {
                        log::warn!(
                            "User {} tried to change name above the allowed character limit",
                            self.id
                        );
                        self.stream
                            .send(ServerMessage::NameTooLong(name).as_wsmsg())
                            .await?;
                        return Ok(false);
                    }
                    match room.get_user_mut(&self.id) {
                        Some(user) => {
                            user.set_name(name);
                            let _ = self.tx.send(ServerMessage::UserNameChange(user.clone()));
                        }
                        None => {
                            self.stream
                                .send(ServerMessage::InvalidUser(self.id).as_wsmsg())
                                .await?;
                        }
                    }
                }
                ClientMessage::GetUserData(uuid) => {
                    if let Some(user) = self.room.lock().await.get_user(&uuid) {
                        self.stream
                            .send(ServerMessage::UserData(user.clone()).as_wsmsg())
                            .await?;
                    } else {
                        self.stream
                            .send(ServerMessage::InvalidUser(uuid).as_wsmsg())
                            .await?;
                    }
                }
                ClientMessage::GetAllUserData => {
                    let room = self.room.lock().await;
                    let mut users = room.get_all_users();
                    users.retain(|u| *u.get_id() != self.id);
                    self.stream
                        .send(ServerMessage::AllUsers(users).as_wsmsg())
                        .await?;
                }
                ClientMessage::GetSelf => {
                    let room = self.room.lock().await;
                    let user = room.get_user(&self.id).expect("Should have self");
                    self.stream
                        .send(ServerMessage::SelfData(user.clone()).as_wsmsg())
                        .await?;
                }
            }
        } else {
            self.send_msg(txt).await?;
        }

        Ok(false)
    }

    fn update_message_counter(&mut self) {
        self.message_counter
            .retain(|i| i.elapsed() < TIMEOUT_WINDOW);
    }

    fn can_send_message(&mut self) -> bool {
        self.update_message_counter();

        self.message_counter.len() < MESSAGE_LIMIT
    }

    fn count_message(&mut self) {
        self.message_counter.push_back(Instant::now());
    }

    async fn send_timeout_message(&mut self) -> WsResult {
        self.stream
            .send(ServerMessage::Timeout(TIMEOUT_DURATION).as_wsmsg())
            .await
    }

    async fn exit_room(&mut self) {
        let mut room = self.room.lock().await;
        if let Some(user) = room.get_user(&self.id) {
            let _ = self.tx.send(ServerMessage::UserLeft(user.clone()));
        }
        room.remove_user(&self.id);
        self.in_room = false;
    }

    async fn close_socket(&mut self) -> WsResult {
        self.exit_room().await;
        self.stream.send(Message::Close(None)).await?;

        Ok(())
    }

    async fn close_logged(&mut self) {
        let _ = self.close_socket().await.inspect_err(|err| {
            log::error!("There was an error while trying to close socket: {err}");
        });
    }

    async fn handle_rx(&mut self, res: Result<ServerMessage, RecvError>) -> WsResult<bool> {
        match res {
            Ok(msg) => {
                self.stream.send(msg.as_wsmsg()).await?;
                log::trace!("User sent: {msg:?}");
                Ok(false)
            }
            Err(broadcast::error::RecvError::Closed) => {
                if let Some(user) = self.room.lock().await.get_user(&self.id) {
                    let _ = self.tx.send(ServerMessage::UserLeft(user.clone()));
                }
                Ok(true)
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("Broadcast lagged behind by {n} messages");
                Ok(false)
            }
        }
    }

    async fn send_msg(&mut self, txt: &str) -> WsResult {
        let txt = self
            .ctx
            .process_with_options(txt.to_string(), &CONTEXT_OPTS);
        match txt {
            Ok(txt) => {
                let _ = self.tx.send(ServerMessage::NewMessage(ChatMessage::new(
                    self.id,
                    txt.clone(),
                )));
            }
            Err(ban) => {
                self.stream
                    .send(
                        ServerMessage::Banned {
                            duration: self.ctx.restricted_for(),
                            reason: ban.generic_str().to_owned(),
                        }
                        .as_wsmsg(),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
