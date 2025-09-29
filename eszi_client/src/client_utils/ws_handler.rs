use eszi_lib::messages::types::{ClientMessage, Message, ServerMessage, User};
use futures::{SinkExt, StreamExt};
use std::sync::mpsc::{Receiver, TryRecvError};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tungstenite::client::IntoClientRequest;

use crate::client_utils::consts::TICK_DURATION;

#[derive(Debug)]
pub enum WsEvent {
    UserAdd(User),
    UserRemove(Uuid),
    UserChange(User),
    Message(Message),
    Quit,
}

#[derive(Debug)]
pub enum WsAction {
    Message(String),
    ChangeName(String),
    Quit,
}

pub struct WsHandler {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: Sender<WsEvent>,
    rx: Receiver<WsAction>,
}

impl WsHandler {
    pub async fn new(tx: Sender<WsEvent>, rx: Receiver<WsAction>) -> Self {
        let url = "ws://127.0.0.1:8000/ws/"
            .into_client_request()
            .expect("Could not parse url");
        let res = connect_async(url).await;
        if res.is_err() {
            let _ = tx.send(WsEvent::Quit).await;
        }
        let (stream, _) = res.expect("Can't continue without websocket connetion");

        Self { stream, tx, rx }
    }

    pub async fn step(&mut self) -> bool {
        if self.handle_actions().await {
            return true;
        }
        let tout = tokio::time::timeout(TICK_DURATION, self.handle_stream());
        match tout.await {
            Ok(res) => res,
            Err(_) => false,
        }
    }

    async fn handle_stream(&mut self) -> bool {
        let msg = self.stream.next().await;
        if msg.is_none() {
            let _ = self.tx.send(WsEvent::Quit).await;
            return true;
        }
        let msg = msg.unwrap();
        match msg {
            Ok(res) => match res {
                tungstenite::Message::Text(txt) => self.handle_message(&txt.to_string()).await,
                tungstenite::Message::Close(_) => {
                    let _ = self.tx.send(WsEvent::Quit);
                    true
                }
                _ => {
                    log::error!("Server trying to user unsupported message types");
                    false
                }
            },
            Err(err) => self.handle_ws_error(err).await,
        }
    }

    async fn handle_actions(&mut self) -> bool {
        let mut recv_res = self.rx.try_recv();
        while let Ok(res) = recv_res {
            match res {
                WsAction::Message(msg) => {
                    let _ = self
                        .stream
                        .send(tungstenite::protocol::Message::Text(
                            ClientMessage::SendMessage(msg).as_json().into(),
                        ))
                        .await;
                    let _ = self.stream.flush().await;
                }
                WsAction::Quit => {
                    let _ = self.stream.close(Default::default()).await;
                    return true;
                }
                WsAction::ChangeName(_) => todo!(),
            }
            recv_res = self.rx.try_recv();
        }
        match recv_res.unwrap_err() {
            TryRecvError::Disconnected => {
                let _ = self.tx.send(WsEvent::Quit);
                return true;
            }
            _ => {}
        }
        return false;
    }

    async fn handle_message(&mut self, txt: &str) -> bool {
        if let Ok(msg) = serde_json::from_str::<ServerMessage>(txt) {
            log::info!(
                "{}",
                serde_json::to_string(&msg).expect("This can not be bad")
            );
            match msg {
                ServerMessage::NewMessage(message) => {
                    let _ = self.tx.send(WsEvent::Message(message)).await;
                    false
                }
                ServerMessage::UserLeft(user) => {
                    let _ = self.tx.send(WsEvent::UserRemove(*user.get_id()));
                    false
                }
                ServerMessage::UserJoined(user) => {
                    let _ = self.tx.send(WsEvent::UserAdd(user));
                    false
                }
                ServerMessage::UserNameChange(user) => {
                    let _ = self.tx.send(WsEvent::UserChange(user));
                    false
                }
                _ => {
                    log::error!("Server returned data it isn't supposed to");
                    true
                }
            }
        } else {
            log::error!("Server trying to send unsupported object or plaint text");
            true
        }
    }

    async fn handle_ws_error(&mut self, err: tungstenite::Error) -> bool {
        match err {
            tungstenite::Error::ConnectionClosed => {
                let _ = self.tx.send(WsEvent::Quit);
                true
            }
            tungstenite::Error::AlreadyClosed => {
                log::warn!("Trying to work with closed websocket");
                let _ = self.tx.send(WsEvent::Quit);
                true
            }
            tungstenite::Error::Io(err) => {
                log::error!("An IO error happened: {err}");
                let _ = self.tx.send(WsEvent::Quit);
                true
            }
            _ => {
                log::error!("Websocket error");
                let _ = self.tx.send(WsEvent::Quit);
                true
            }
        }
    }
}
