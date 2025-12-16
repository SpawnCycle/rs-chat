use chat_lib::prelude::*;
use futures::{SinkExt, StreamExt};
use std::sync::mpsc::{Receiver, TryRecvError};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

use crate::{config::file::AppConfig, consts::TICK_DURATION};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WsEvent {
    SelfInfo(User),
    UserAdd(User),
    UserInfo(User),
    UserChange(User),
    UserRemove(Uuid),
    Message(Message),
    Quit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WsAction {
    Message(String),
    ChangeName(String),
    RequestUser(Uuid),
    RequestSelf,
    Quit,
}

pub struct WsHandler {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: Sender<WsEvent>,
    rx: Receiver<WsAction>,
}

impl WsHandler {
    pub async fn new(
        tx: Sender<WsEvent>,
        rx: Receiver<WsAction>,
        cfg: AppConfig,
    ) -> Result<Self, tungstenite::Error> {
        let (stream, _res) = connect_async(cfg.url.to_string()).await?;

        Ok(Self { stream, tx, rx })
    }

    pub async fn step(&mut self) -> bool {
        if self.process_actions().await {
            return true;
        }
        let tout = tokio::time::timeout(TICK_DURATION / 2, self.handle_stream());
        tout.await.unwrap_or_default()
    }

    pub async fn close(&mut self) {
        let _ = self.stream.close(None).await;
        let _ = self.tx.send(WsEvent::Quit).await;
    }

    async fn handle_stream(&mut self) -> bool {
        let msg = self.stream.next().await;
        match msg {
            None => {
                self.close().await;
                true
            }
            Some(Ok(res)) => match res {
                tungstenite::Message::Text(txt) => self.handle_message(txt.as_ref()).await,
                tungstenite::Message::Close(_) => {
                    self.close().await;
                    true
                }
                _ => {
                    log::error!("Server trying to user unsupported message types");
                    false
                }
            },
            Some(Err(err)) => self.handle_ws_error(err).await,
        }
    }

    async fn process_actions(&mut self) -> bool {
        let mut recv = self.rx.try_recv();
        while let Ok(res) = &recv {
            self.handle_action(res).await;
            recv = self.rx.try_recv();
        }
        if self.rx.try_recv().expect_err("Already checked") == TryRecvError::Disconnected {
            self.close().await;
            return true;
        }
        false
    }

    async fn handle_action(&mut self, res: &WsAction) -> bool {
        match res {
            WsAction::Message(msg) => {
                let _ = self
                    .stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::SendMessage(msg.clone()).as_json().into(),
                    ))
                    .await;
                let _ = self.stream.flush().await;
            }
            WsAction::Quit => {
                let _ = self.stream.close(None).await;
                return true;
            }
            WsAction::ChangeName(name) => {
                let _ = self
                    .stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::ChangeUserName(name.clone()).as_json().into(),
                    ))
                    .await;
            }
            WsAction::RequestUser(uuid) => {
                let _ = self
                    .stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::GetUserData(*uuid).as_json().into(),
                    ))
                    .await;
            }
            WsAction::RequestSelf => {
                let _ = self
                    .stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::GetSelf.as_json().into(),
                    ))
                    .await;
            }
        }
        false
    }

    async fn handle_message(&mut self, txt: &str) -> bool {
        if let Ok(msg) = serde_json::from_str::<ServerMessage>(txt) {
            log::trace!(
                "{}",
                serde_json::to_string(&msg).expect("This can not be bad")
            );
            match msg {
                ServerMessage::NewMessage(message) => {
                    let _ = self.tx.send(WsEvent::Message(message)).await;
                    false
                }
                ServerMessage::UserLeft(user) => {
                    let _ = self.tx.send(WsEvent::UserRemove(*user.get_id())).await;
                    false
                }
                ServerMessage::UserJoined(user) => {
                    let _ = self.tx.send(WsEvent::UserAdd(user)).await;
                    false
                }
                ServerMessage::UserNameChange(user) => {
                    let _ = self.tx.send(WsEvent::UserChange(user)).await;
                    false
                }
                ServerMessage::SelfData(user) => {
                    let _ = self.tx.send(WsEvent::SelfInfo(user)).await;
                    false
                }
                ServerMessage::UserData(user) => {
                    let _ = self.tx.send(WsEvent::UserInfo(user)).await;
                    false
                }
                _ => {
                    // TODO: implement these
                    log::error!("Server sending unimplemented data: {msg:?}");
                    false
                }
            }
        } else {
            log::error!("Server trying to send unsupported object or plaint text");
            false
        }
    }

    async fn handle_ws_error(&mut self, err: tungstenite::Error) -> bool {
        match err {
            tungstenite::Error::ConnectionClosed => {
                self.close().await;
                true
            }
            tungstenite::Error::AlreadyClosed => {
                log::warn!("Trying to work with closed websocket");
                self.close().await;
                true
            }
            tungstenite::Error::Io(err) => {
                log::error!("An IO error happened: {err}");
                self.close().await;
                true
            }
            _ => {
                log::error!("Websocket error");
                self.close().await;
                true
            }
        }
    }
}
