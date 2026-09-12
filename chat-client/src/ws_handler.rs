use std::{
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use anyhow::{Context, anyhow};
use chat_lib::{Version, prelude::*, ws_connection::WsConnection};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite};
use url::{Url, form_urlencoded};
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
    /// The amount of timeout added in seconds
    TimeoutAdded(u64),
    Quit,

    /// A non-fatal error that should still be reported
    SoftError(String),
    /// A fatal error
    FatalError(String),
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
    closed: bool,
}

impl WsHandler {
    /// # Errors
    ///
    /// This function returns an error if there was a problem connecting to the websocket
    ///
    /// # Panics
    ///
    /// This function panics if any of the default values are incorrect
    pub async fn connect(
        tx: Sender<WsEvent>,
        rx: Receiver<WsAction>,
        config: WebConfig,
        room: String,
        mut url: Url,
        version: Version,
        initial_name: Option<String>,
    ) -> anyhow::Result<Self> {
        // TODO: Better error reporting/handling instread of just using anyhow
        if url.scheme() == "https" {
            url.set_scheme("wss").expect("The url should be correct");
        } else {
            url.set_scheme("ws").expect("The url should be correct");
        }

        let mut url = url
            .join(&format!("{version}/room/{room}"))
            .context("Couldn't parse url string")?;

        if let Some(name) = initial_name {
            let name = form_urlencoded::byte_serialize(name.as_bytes()).collect::<String>();
            url.set_query(Some(&format!("name={name}")));
        }

        log::debug!("Trying to connect to websocket {url}");

        let stream = Self::connect_websocket(&url).await;

        if let Err(err) = &stream {
            let _ = tx.send(WsEvent::FatalError(err.to_string())).await;
            let _ = tx.send(WsEvent::Quit).await;
            log::error!("Could not connect to server websocket: {err}");
        }
        let stream = stream?;

        Ok(Self::new(config, stream, tx, rx))
    }

    pub const fn new(
        config: WebConfig,
        stream: WsConnection,
        tx: Sender<WsEvent>,
        rx: Receiver<WsAction>,
    ) -> Self {
        Self {
            config,
            stream,
            tx,
            rx,
            closed: false,
        }
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

    /// Processes all of the incoming and outgoing information,
    /// returnes true if the stream is closed and should no longer be polled
    pub async fn step(&mut self) -> bool {
        if self.closed {
            return true;
        }

        let mut should_quit = self.process_actions().await;

        tokio::select! {
            res = self.handle_stream() => {
                let res = res.inspect_err(|err| log::error!("{err}")).unwrap_or(true);
                should_quit = should_quit || res;
            }
            () = tokio::time::sleep(TICK_DURATION / 2) => {}
        };

        should_quit
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        if !self.closed {
            self.client_close().await?;
        }

        Ok(())
    }

    async fn send_close(&mut self) -> anyhow::Result<()> {
        log::info!("Closing Ws stream");
        let _ = self.tx.send(WsEvent::Quit).await;
        self.stream.close().await?;
        self.stream.flush().await?;

        self.closed = true;

        Ok(())
    }

    async fn client_close(&mut self) -> anyhow::Result<()> {
        self.send_close().await?;

        // a close is only valid if both of the parties sent a close frame,
        self.await_close_frame().await;
        log::info!("Received Close frame");

        Ok(())
    }

    async fn server_close(&mut self) -> anyhow::Result<()> {
        self.send_close().await?;

        Ok(())
    }

    async fn await_close_frame(&mut self) {
        while let Some(msg) = self.stream.next().await {
            let Ok(msg) = msg else {
                break;
            };
            // we need to await a close frame (or an error),
            // so the close can be finalize properly
            if matches!(msg, tungstenite::Message::Close(_)) {
                break;
            }
        }
    }

    async fn handle_stream(&mut self) -> anyhow::Result<bool> {
        let msg = self.stream.next().await;

        let msg = match msg {
            Some(msg) => msg?,
            None => return Ok(false),
        };

        match msg {
            tungstenite::Message::Text(txt) => {
                self.handle_message(txt.as_ref()).await?;
            }
            tungstenite::Message::Close(_) => {
                self.server_close().await?;
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
                self.client_close().await?;
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
                self.send_event(WsEvent::Message(message)).await;
            }
            ServerMessage::UserLeft(user) => {
                self.send_event(WsEvent::UserRemove(*user.get_id())).await;
            }
            ServerMessage::UserJoined(user) => {
                self.send_event(WsEvent::UserAdd(user)).await;
            }
            ServerMessage::UserNameChange(user) => {
                self.send_event(WsEvent::UserChange(user)).await;
            }
            ServerMessage::SelfData(user) => {
                self.send_event(WsEvent::SelfInfo(user)).await;
            }
            ServerMessage::UserData(user) => {
                self.send_event(WsEvent::UserInfo(user)).await;
            }
            ServerMessage::Banned { duration, reason } => {
                self.send_event(WsEvent::Banned(duration, reason)).await;
            }
            ServerMessage::AllUsers(users) => {
                self.send_event(WsEvent::AllUserInfo(users)).await;
            }
            ServerMessage::TimeoutAdded(secs) => {
                self.send_event(WsEvent::TimeoutAdded(secs)).await;
            }
            ServerMessage::InvalidUser(id) => {
                self.send_event(WsEvent::SoftError(format!(
                    "Tried to get a user that doesn't exist: {id}"
                )))
                .await;
            }
            ServerMessage::UnsupportedMessage(err) | ServerMessage::NameTooLong(err) => {
                self.send_event(WsEvent::SoftError(err)).await;
            }
            ServerMessage::NameInappropriate => {
                self.send_event(WsEvent::SoftError(
                    "Tried to change name to an inappropriate one".to_string(),
                ))
                .await;
            }
            ServerMessage::Heartbeat => {
                // Nothing needs to be done
            }
        }

        Ok(())
    }

    async fn send_event(&mut self, event: WsEvent) {
        let _ = self.tx.send(event).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chat_lib::ws_connection::Message;

    use crate::testing::{ConfiguredHandler, handler_run, rx_collect_available};

    #[tokio::test]
    async fn handler_exits() -> anyhow::Result<()> {
        let ConfiguredHandler {
            mut handler,
            mut e_rx,
            in_tx,
            mut out_rx,
            ..
        } = ConfiguredHandler::new();

        in_tx.send(tungstenite::Message::Close(None)).await?;

        handler_run(&mut handler, 5).await;

        let out_vec = rx_collect_available(&mut out_rx);
        let ev_vec = rx_collect_available(&mut e_rx);

        assert!(out_vec.contains(&Message::Close(None)));
        assert!(ev_vec.contains(&WsEvent::Quit));

        Ok(())
    }

    #[tokio::test]
    async fn sends_message() -> anyhow::Result<()> {
        let ConfiguredHandler {
            mut handler,
            a_tx,
            mut out_rx,
            ..
        } = ConfiguredHandler::new();

        let message_text = "Hello";

        a_tx.send(WsAction::Message(message_text.to_string()))?;

        handler_run(&mut handler, 3).await;

        let out_vec = rx_collect_available(&mut out_rx);

        assert!(out_vec.contains(&ClientMessage::SendMessage(message_text.to_string()).as_wsmsg()));

        Ok(())
    }

    #[tokio::test]
    async fn quits() -> anyhow::Result<()> {
        let ConfiguredHandler {
            mut handler,
            a_tx,
            mut out_rx,
            in_tx,
            ..
        } = ConfiguredHandler::new();

        let run_steps = 3;

        let n = handler_run(&mut handler, run_steps).await;

        // the handler should run the full `run_steps` because it's not closed during those polls
        assert_eq!(n, run_steps);

        a_tx.send(WsAction::Quit)?;

        let n = handler_run(&mut handler, 3).await;

        assert_eq!(n, 1);

        in_tx.send(Message::Close(None)).await?;

        let n = handler_run(&mut handler, 3).await;

        assert_eq!(n, 1);

        let out_vec = rx_collect_available(&mut out_rx);

        // check if the handler sent a closing frame back
        assert!(out_vec.contains(&Message::Close(None)));

        Ok(())
    }
}
