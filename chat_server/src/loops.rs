use chat_lib::prelude::*;
use chat_lib::types::Message as ChatMessage;
use chat_lib::types::Sync;

use rocket::futures::{SinkExt, StreamExt};
use rocket_ws::{Message, result, stream::DuplexStream};
use rustrict::Context;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use crate::types::Room;
use crate::types::{MsgBroadcastReceiver, MsgBroadcastSender};

pub type WsLoopResult = Result<bool, result::Error>;

pub struct WsLoopCtx {
    rx: MsgBroadcastReceiver,
    tx: MsgBroadcastSender,
    stream: DuplexStream,
    room: Sync<Room>,
    id: Uuid,
    ctx: Context,
}

impl WsLoopCtx {
    pub const fn new(
        rx: MsgBroadcastReceiver,
        tx: MsgBroadcastSender,
        stream: DuplexStream,
        room: Sync<Room>,
        id: Uuid,
        ctx: Context,
    ) -> Self {
        Self {
            rx,
            tx,
            stream,
            room,
            id,
            ctx,
        }
    }

    pub async fn ws_step(&mut self) -> WsLoopResult {
        tokio::select! {
            Some(res) = self.stream.next() => {
                match res {
                    Err(err) => {
                        self.handle_stream_err().await?;

                        Err(err)
                    }
                    Ok(msg) => {
                        match msg {
                            Message::Text(txt) => self.handle_text(&txt).await,
                            Message::Close(_) => {
                                if let Some(user) = self.room.lock().await.get_user(self.id) {
                                    let _ = self.tx.send(ServerMessage::UserLeft(user.clone()));
                                }
                                Ok(true)
                            },
                            _ => Ok(true)
                        }
                    }
                }
            }
            res = self.rx.recv() => self.handle_rx(res).await,
            else => {
                if let Some(user) = self.room.lock().await.get_user(self.id) {
                    let _ = self.tx.send(ServerMessage::UserLeft(user.clone()));
                }
                Ok(true)
            }
        }
    }

    async fn handle_text(&mut self, txt: &str) -> WsLoopResult {
        if let Ok(msg) = serde_json::from_str::<ClientMessage>(txt) {
            match msg {
                ClientMessage::SendMessage(msg) => {
                    self.send_msg(&msg).await?;
                }
                ClientMessage::ChangeUserName(name) => {
                    let mut room = self.room.lock().await;
                    match room.get_user_mut(self.id) {
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
                    if let Some(user) = self.room.lock().await.get_user(uuid) {
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
                    let user = room
                        .users
                        .iter()
                        .find(|u| *u.get_id() == self.id)
                        .expect("Should have self");
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

    async fn handle_rx(&mut self, res: Result<ServerMessage, RecvError>) -> WsLoopResult {
        match res {
            Ok(msg) => {
                self.stream.send(msg.as_wsmsg()).await?;
                log::info!("User sent: {:?}", msg);
                Ok(false)
            }
            Err(broadcast::error::RecvError::Closed) => {
                if let Some(user) = self.room.lock().await.get_user(self.id) {
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

    async fn handle_stream_err(&mut self) -> Result<(), result::Error> {
        let mut room = self.room.lock().await;
        let user = room.get_user(self.id);
        self.stream.close(Default::default()).await?;
        if let Some(user) = user {
            let _ = self.tx.send(ServerMessage::UserLeft(user.clone()));
        }
        room.remove_user(self.id);

        Ok(())
    }

    async fn send_msg(&mut self, txt: &str) -> Result<(), result::Error> {
        let txt = self.ctx.process(txt.to_string());
        match txt {
            Ok(txt) => {
                let _ = self.tx.send(ServerMessage::NewMessage(ChatMessage::new(
                    self.id,
                    txt.to_string(),
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
