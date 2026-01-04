use chat_lib::prelude::*;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use std::sync::mpsc::{Receiver, TryRecvError};
use tokio::{net::TcpStream, sync::mpsc::Sender, time::timeout};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};
use url::Url;
use uuid::Uuid;

use crate::{
    config::file::WebConfig,
    consts::{TICK_DURATION, WS_TIMEOUT_DURATION},
};

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

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct WsHandler {
    stream: WsStream,
    tx: Sender<WsEvent>,
    rx: Receiver<WsAction>,
}

impl WsHandler {
    pub async fn new(
        tx: Sender<WsEvent>,
        rx: Receiver<WsAction>,
        cfg: WebConfig,
    ) -> anyhow::Result<Self> {
        // TODO: Better error reporting/handling instread of just using anyhow
        let stream = Self::connect(&cfg.url).await;
        // would inspect_err, but I need to await the sender
        if let Err(err) = &stream {
            let _ = tx.send(WsEvent::Quit).await;
            error!("Could not connect to server websocket: {err}");
        }
        let stream = stream?;

        Ok(Self { stream, tx, rx })
    }

    async fn connect(cfg: &Url) -> anyhow::Result<WsStream> {
        let tout = timeout(WS_TIMEOUT_DURATION, connect_async(cfg.to_string()));
        let (stream, _res) = tout.await??;

        Ok(stream)
    }

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

    async fn handle_stream(&mut self) -> bool {
        let msg = self.stream.next().await;
        match msg {
            None => {
                debug!("stream resolved to None: {msg:?}");
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

    async fn process_actions(&mut self) -> bool {
        let mut actions = self.rx.try_iter().collect::<Vec<_>>();
        if actions.is_empty() {
            match self.rx.try_recv() {
                Ok(v) => actions.push(v),
                Err(TryRecvError::Disconnected) => {
                    error!("App rx disconnected before sending a program end signal");
                    return true;
                }
                // This is fine
                Err(TryRecvError::Empty) => {}
            }
        }
        for res in &actions {
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
        }
        false
    }

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
