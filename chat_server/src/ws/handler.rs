use chat_lib::{
    prelude::*,
    types::{Message as ChatMessage, Sync},
};
use rocket::{
    futures::{SinkExt, StreamExt},
    serde::json::serde_json,
};
use rocket_ws::{
    result::Error,
    {Message, stream::DuplexStream},
};
use rustrict::Context;
use tokio::sync::broadcast::{self, error::RecvError};
use uuid::Uuid;

use crate::config::CONTEXT_OPTS;
use crate::ws::{MsgBroadcastReceiver, MsgBroadcastSender, Room};

pub type WsResult<T = ()> = Result<T, rocket_ws::result::Error>;

pub struct WsHandler<'a, F>
where
    F: Future<Output = ()> + Clone,
{
    stream: DuplexStream,
    ctx: Context,
    id: Uuid,
    rx: MsgBroadcastReceiver,
    tx: MsgBroadcastSender,
    room: Sync<Room>,
    sd: &'a mut F,
    in_room: bool,
}

impl<'a, F> WsHandler<'a, F>
where
    F: Future<Output = ()> + Clone,
{
    pub const fn new(
        stream: DuplexStream,
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
            id,
            rx,
            tx,
            room,
            sd,
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

    async fn handle_stream(&mut self, res: Result<Message, Error>) -> WsResult<bool> {
        match res {
            Err(err) => {
                self.close_socket().await?;
                Err(err)
            }
            Ok(msg) => match msg {
                Message::Text(txt) => self.handle_text(&txt).await,
                Message::Close(_) => {
                    self.exit_room().await;
                    Ok(true)
                }
                _ => {
                    self.close_logged().await;
                    Ok(true)
                }
            },
        }
    }

    async fn handle_text(&mut self, txt: &str) -> WsResult<bool> {
        if let Ok(msg) = serde_json::from_str::<ClientMessage>(txt) {
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
        self.stream.close(None).await?;

        Ok(())
    }

    async fn close_logged(&mut self) {
        let _ = self.close_socket().await.inspect_err(|err| {
            log::error!("There was an error while trying to close socket: {err}",);
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
