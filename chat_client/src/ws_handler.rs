use anyhow::{Context, anyhow};
use chat_lib::prelude::*;
use futures::{SinkExt, StreamExt};
use std::{
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};
use tokio::{net::TcpStream, sync::mpsc::Sender};
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
    Banned(Duration, String),
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

/// The type that stands between the server and the client,
/// handling the communication using types `WsEvent` and `WsAction`
/// needs to be closed manually
#[derive(Debug)]
pub struct WsHandler {
    stream: WsStream,
    tx: Sender<WsEvent>,
    rx: Receiver<WsAction>,
}

fn log_ws_error(err: &tungstenite::Error) {
    match err {
        tungstenite::Error::AlreadyClosed => {
            log::warn!("Trying to work with closed websocket");
        }
        _ => {
            log::error!("Websocket error: {err:?}");
        }
    }
}

impl WsHandler {
    /// # Errors
    ///
    /// This function returns an error if there was a problem connecting to the websocket
    ///
    /// # Panics
    ///
    /// This function panics if any of the default values are incorrect
    pub async fn new(
        tx: Sender<WsEvent>,
        rx: Receiver<WsAction>,
        cfg: WebConfig,
        room: String,
    ) -> anyhow::Result<Self> {
        // TODO: Better error reporting/handling instread of just using anyhow
        let mut url = cfg.url.clone();
        if url.scheme() == "https" {
            url.set_scheme("wss").expect("The url should be correct");
        } else {
            url.set_scheme("ws").expect("The url should be correct");
        }

        let stream = Self::connect_websocket(
            &url.join(&format!("room/{room}"))
                .context("Couldn't parse url string")?,
        )
        .await;

        if let Err(err) = &stream {
            let _ = tx.send(WsEvent::Quit).await;
            log::error!("Could not connect to server websocket: {err}");
        }
        let stream = stream?;

        Ok(Self { stream, tx, rx })
    }

    async fn connect_websocket(cfg: &Url) -> anyhow::Result<WsStream> {
        tokio::select! {
            conn = connect_async(cfg.to_string()) => {
                let (stream, _res) = conn?;
                Ok(stream)
            }
            () = tokio::time::sleep(WS_TIMEOUT_DURATION) => {
                Err(anyhow!("The connection was taking too long"))
            }
        }
    }

    pub async fn step(&mut self) -> bool {
        let mut should_quit = self.process_actions().await;

        tokio::select! {
            res = self.handle_stream() => {
                let res = res.inspect_err(|err| log::error!("{err}")).unwrap_or(true);
                should_quit = should_quit || res;
            }
            () = tokio::time::sleep(TICK_DURATION / 2) => {
            }
        };

        should_quit
    }

    pub async fn close(&mut self) {
        log::info!("Closing Ws stream");
        let _ = self.tx.send(WsEvent::Quit).await;
        let _ = self.stream.close(None).await;
    }

    async fn handle_stream(&mut self) -> anyhow::Result<bool> {
        let msg = self
            .stream
            .next()
            .await
            .context("stream resolved to None")??;

        match msg {
            tungstenite::Message::Text(txt) => {
                self.handle_message(txt.as_ref()).await?;
            }
            tungstenite::Message::Close(_) => {
                return Ok(true);
            }
            _ => {
                log::error!("Server trying to user unsupported message types");
                return Ok(false);
            }
        }

        Ok(false)
    }

    async fn process_actions(&mut self) -> bool {
        let mut actions = self.rx.try_iter().collect::<Vec<_>>();
        if actions.is_empty() {
            match self.rx.try_recv() {
                Ok(v) => actions.push(v),
                Err(TryRecvError::Disconnected) => {
                    log::error!("App rx disconnected before sending a program end signal");
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
                    log_ws_error(&err);
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

    async fn handle_message(&mut self, txt: &str) -> anyhow::Result<()> {
        let msg = serde_json::from_str::<ServerMessage>(txt).map_err(|err| {
            anyhow!("Server trying to send unsupported object or plaint text: {err} : {txt}")
        })?;

        log::trace!("Server Message: {txt}");

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
            ServerMessage::Banned { duration, reason } => {
                let _ = self.tx.send(WsEvent::Banned(duration, reason)).await;
            }
            ServerMessage::AllUsers(_)
            | ServerMessage::UnsupportedMessage(_)
            | ServerMessage::InvalidUser(_)
            | ServerMessage::NameTooLong(_) => {
                // TODO: implement these
                log::error!("Server sending unimplemented data: {msg:?}");
            }
        }

        Ok(())
    }
}
