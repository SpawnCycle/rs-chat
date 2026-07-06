use std::{
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use anyhow::{Context, anyhow};
use chat_lib::{prelude::*, ws_connection::WsConnection};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite};
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
    AllUserInfo(Vec<User>),
    UserInfo(User),
    UserChange(User),
    UserRemove(Uuid),
    Message(Message),
    Banned(Duration, String),
    Quit,

    Error(String),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum WsAction {
    Message(String),
    ChangeName(String),
    RequestUser(Uuid),
    RequestAll,
    RequestSelf,
    Quit,
}

/// The type that stands between the server and the client,
/// handling the communication using types `WsEvent` and `WsAction`
/// needs to be closed manually
#[derive(Debug)]
pub struct WsHandler {
    #[allow(unused)]
    config: WebConfig,
    stream: WsConnection,
    tx: Sender<WsEvent>,
    rx: Receiver<WsAction>,
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
        config: WebConfig,
        room: String,
        mut url: Url,
    ) -> anyhow::Result<Self> {
        // TODO: Better error reporting/handling instread of just using anyhow
        if url.scheme() == "https" {
            url.set_scheme("wss").expect("The url should be correct");
        } else {
            url.set_scheme("ws").expect("The url should be correct");
        }

        let url = &url
            .join(&format!("room/{room}"))
            .context("Couldn't parse url string")?;

        log::debug!("Trying to connect to websocket {url}");

        let stream = Self::connect_websocket(url).await;

        if let Err(err) = &stream {
            let _ = tx.send(WsEvent::Quit).await;
            log::error!("Could not connect to server websocket: {err}");
        }
        let stream = stream?;

        Ok(Self {
            config,
            stream,
            tx,
            rx,
        })
    }

    async fn connect_websocket(cfg: &Url) -> anyhow::Result<WsConnection> {
        tokio::select! {
            conn = connect_async(cfg.to_string()) => {
                let (stream, _res) = conn?;
                Ok(WsConnection::from(stream))
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
        let _ = self.stream.flush().await;
        let _ = self.stream.close().await;
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
                log::error!("User sent an unsupported message type");
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

        let mut should_exit = false;
        for res in &actions {
            match self.handle_action(res).await {
                Ok(exit) => {
                    should_exit |= exit;
                }
                Err(err) => {
                    log::error!("{err}");
                    should_exit = true;
                }
            }
        }

        should_exit
    }

    async fn handle_action(&mut self, msg: &WsAction) -> anyhow::Result<bool> {
        match msg {
            WsAction::Message(msg) => {
                self.stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::SendMessage(msg.clone()).as_json().into(),
                    ))
                    .await?;
            }
            WsAction::Quit => {
                self.stream.flush().await?;
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
            WsAction::RequestAll => {
                self.stream
                    .send(tungstenite::Message::Text(
                        ClientMessage::GetAllUserData.as_json().into(),
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

        log::debug!("Server Message: {msg:?}");

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
            ServerMessage::AllUsers(users) => {
                let _ = self.tx.send(WsEvent::AllUserInfo(users)).await;
            }
            ServerMessage::UnsupportedMessage(_)
            | ServerMessage::TimeoutAdded(_)
            | ServerMessage::InvalidUser(_)
            | ServerMessage::NameTooLong(_) => {
                // TODO: implement these
                log::error!("Server sending unimplemented data: {msg:?}");
            }
            ServerMessage::Heartbeat => {
                // Nothing needs to be done
            }
        }

        Ok(())
    }
}
