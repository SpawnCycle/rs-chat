use chat_lib::prelude::*;
use futures::{SinkExt, StreamExt};
use std::sync::mpsc::{Receiver, TryRecvError};
use tokio::{net::TcpStream, sync::mpsc::Sender, time::timeout};
use tracing::{Level, debug, error, info, instrument, trace, warn};
use uuid::Uuid;

use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

use crate::consts::WS_TIMEOUT_DURATION;
use crate::{config::file::AppConfig, consts::TICK_DURATION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsEvent {
    SelfInfo(User),
    UserAdd(User),
    UserInfo(User),
    UserChange(User),
    UserRemove(Uuid),
    Message(Message),
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsAction {
    Message(String),
    ChangeName(String),
    RequestUser(Uuid),
    RequestSelf,
    Quit,
}

#[derive(Debug)]
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
    ) -> Result<Self, anyhow::Error> {
        let tout = timeout(WS_TIMEOUT_DURATION, connect_async(cfg.url.to_string()));
        let (stream, _res) = tout.await??;

        Ok(Self { stream, tx, rx })
    }

    #[instrument(skip_all, level = Level::DEBUG)]
    pub async fn step(&mut self) -> bool {
        let res = self.process_actions().await;
        let tout = tokio::time::timeout(TICK_DURATION / 2, self.handle_stream());
        res || tout.await.unwrap_or_default()
    }

    pub async fn close(&mut self) {
        info!("Closing Ws stream");
        let _ = self.tx.send(WsEvent::Quit).await;
        let _ = self.stream.close(None).await;
    }

    #[instrument(skip_all, level = Level::DEBUG)]
    async fn handle_stream(&mut self) -> bool {
        let msg = self.stream.next().await;
        match msg {
            None => {
                debug!(?msg, "stream resolved to None");
                self.close().await;
                true
            }
            Some(Ok(res)) => match res {
                tungstenite::Message::Text(txt) => {
                    self.handle_message(txt.as_ref()).await;
                    false
                }
                tungstenite::Message::Close(_) => {
                    self.close().await;
                    true
                }
                _ => {
                    error!("Server trying to user unsupported message types");
                    false
                }
            },
            Some(Err(err)) => {
                self.handle_ws_error(err).await;
                true
            }
        }
    }

    #[instrument(skip_all, level = Level::DEBUG)]
    async fn process_actions(&mut self) -> bool {
        let mut recv = self.rx.try_recv();
        while let Ok(res) = &recv {
            match self.handle_action(res).await {
                Ok(exit) if exit => {
                    return true;
                }
                Err(err) => {
                    self.handle_ws_error(err).await;
                    return true;
                }
                _ => {}
            }
            recv = self.rx.try_recv();
        }
        if recv.expect_err("Already checked") == TryRecvError::Disconnected {
            self.close().await;
            return true;
        }
        false
    }

    #[instrument(skip(self), level = Level::DEBUG)]
    async fn handle_action(&mut self, msg: &WsAction) -> tungstenite::Result<bool> {
        match msg {
            WsAction::Message(msg) => {
                self.stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::SendMessage(msg.clone()).as_json().into(),
                    ))
                    .await?;
            }
            WsAction::Quit => {
                self.close().await;
                return Ok(true);
            }
            WsAction::ChangeName(name) => {
                self.stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::ChangeUserName(name.clone()).as_json().into(),
                    ))
                    .await?;
            }
            WsAction::RequestUser(uuid) => {
                self.stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::GetUserData(*uuid).as_json().into(),
                    ))
                    .await?;
            }
            WsAction::RequestSelf => {
                self.stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::GetSelf.as_json().into(),
                    ))
                    .await?;
            }
        }
        Ok(false)
    }

    async fn handle_message(&mut self, txt: &str) {
        if let Ok(msg) = serde_json::from_str::<ServerMessage>(txt) {
            trace!(
                "{}",
                serde_json::to_string(&msg).expect("This can not be bad")
            );
            match msg {
                ServerMessage::NewMessage(message) => {
                    let _ = self.tx.send(WsEvent::Message(message)).await;
                }
                ServerMessage::UserLeft(user) => {
                    let _ = self.tx.send(WsEvent::UserRemove(*user.get_id())).await;
                }
                ServerMessage::UserJoined(user) => {
                    let _ = self.tx.send(WsEvent::UserAdd(user)).await;
                }
                ServerMessage::UserNameChange(user) => {
                    let _ = self.tx.send(WsEvent::UserChange(user)).await;
                }
                ServerMessage::SelfData(user) => {
                    let _ = self.tx.send(WsEvent::SelfInfo(user)).await;
                }
                ServerMessage::UserData(user) => {
                    let _ = self.tx.send(WsEvent::UserInfo(user)).await;
                }
                _ => {
                    // TODO: implement these
                    error!("Server sending unimplemented data: {msg:?}");
                }
            }
        } else {
            error!("Server trying to send unsupported object or plaint text");
        }
    }

    async fn handle_ws_error(&mut self, err: tungstenite::Error) {
        match err {
            tungstenite::Error::AlreadyClosed => {
                warn!("Trying to work with closed websocket");
            }
            tungstenite::Error::Io(err) => {
                error!("An IO error happened: {err}");
            }
            _ => {
                error!("Websocket error");
            }
        }
        self.close().await;
    }
}
