use std::collections::HashMap;

use better_eszichat::messages::types::{Message, Sync, User, WsMessage};
use futures::SinkExt;
use rocket::futures::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use uuid::Uuid;

use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tungstenite::client::IntoClientRequest;

use crate::TEST_UUID;

pub enum WsAction {
    UserAdd(User),
    UserRemove(Uuid),
    UserChange(User),
    Message(Message),
    Quit,
}

pub struct WsHandler {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: mpsc::Sender<WsAction>,
}

impl WsHandler {
    pub async fn new(tx: mpsc::Sender<WsAction>) -> Self {
        let url = "ws://127.0.0.1:8000/ws/"
            .into_client_request()
            .expect("Could not parse url");
        let res = connect_async(url).await;
        if res.is_err() {
            let _ = tx.send(WsAction::Quit).await;
        }
        let (stream, _) = res.expect("Can't continue without websocket connetion");

        Self { stream, tx }
    }

    pub async fn step(&mut self) -> bool {
        let msg = self.stream.next().await;
        if msg.is_none() {
            let _ = self.tx.send(WsAction::Quit).await;
            return true;
        }
        let msg = msg.unwrap();
        match msg {
            Ok(res) => match res {
                tungstenite::Message::Text(txt) => {
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&txt) {
                        log::info!(
                            "{}",
                            serde_json::to_string(&msg).expect("This can not be bad")
                        );
                        match msg {
                            WsMessage::NewMessage(message) => {
                                let _ = self.tx.send(WsAction::Message(message)).await;
                                false
                            }
                            WsMessage::UserLeft(user) => {
                                let _ = self.tx.send(WsAction::UserRemove(*user.get_id()));
                                false
                            }
                            WsMessage::UserJoined(user) => {
                                let _ = self.tx.send(WsAction::UserAdd(user));
                                false
                            }
                            WsMessage::UserNameChange(user) => {
                                let _ = self.tx.send(WsAction::UserChange(user));
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
                tungstenite::Message::Close(_) => {
                    let _ = self.tx.send(WsAction::Quit);
                    true
                }
                _ => {
                    log::error!("Server trying to user unsupported message types");
                    false
                }
            },
            Err(err) => match err {
                tungstenite::Error::ConnectionClosed => {
                    let _ = self.tx.send(WsAction::Quit);
                    true
                }
                tungstenite::Error::AlreadyClosed => {
                    log::warn!("Trying to work with closed websocket");
                    let _ = self.tx.send(WsAction::Quit);
                    true
                }
                tungstenite::Error::Io(err) => {
                    log::error!("An IO error happened: {err}");
                    let _ = self.tx.send(WsAction::Quit);
                    true
                }
                _ => {
                    log::error!("Websocket error");
                    let _ = self.tx.send(WsAction::Quit);
                    true
                }
            },
        }
    }
}
