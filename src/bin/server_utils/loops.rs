use better_eszichat::messages::types::Message as ChatMessage;
use better_eszichat::messages::types::WsMessage;
use rocket::futures::{SinkExt, StreamExt};
use rocket_ws::{
    Message,
    result::{self, Error},
    stream::DuplexStream,
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::server_utils::types::{MsgBroadcastReceiver, MsgBroadcastSender};

pub async fn ws_loop(
    id: Uuid,
    stream: &mut DuplexStream,
    rx: &mut MsgBroadcastReceiver,
    tx: &MsgBroadcastSender,
) -> Result<bool, result::Error> {
    tokio::select! {
        Some(res) = stream.next() => {
            match res {
                Err(err) => {
                    match &err {
                        Error::ConnectionClosed => return Ok(true),
                        Error::AlreadyClosed => return Ok(true),
                        _ => {}
                    };
                    return Err(err);
                }
                Ok(msg) => {
                    match msg {
                        Message::Text(txt)=>{
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&txt) {
                                match msg {
                                    WsMessage::NewMessage(message) => todo!(),
                                    WsMessage::UserNameChange(user) => todo!(),
                                    WsMessage::GetUserName(uuid) => todo!(),
                                    WsMessage::Arbitrary(_) => todo!(),
                                    _ => unreachable!()
                                }
                            } else {
                                let _ = tx.send(WsMessage::NewMessage(ChatMessage::new(id, txt)));
                            }
                            Ok(false)
                        }
                        Message::Close(_) => Ok(true),
                        _ => todo!()
                    }
                }
            }
        }
        res = rx.recv() => {
            match res {
                Ok(msg)=>{
                    if let Some(false) = msg.is_user(id) {
                        stream.send(msg.as_wsmsg()).await?;
                    }
                    Ok(false)
                }
                Err(broadcast::error::RecvError::Closed) => Ok(true),
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("Broadcast lagged behind by {n} messages");
                    Ok(false)
                }
            }
        }
        else => Ok(true)
    }
}
