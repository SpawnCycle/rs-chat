use eszi_lib::types::ClientMessage;
use eszi_lib::types::Message as ChatMessage;
use eszi_lib::types::ServerMessage;
use eszi_lib::types::Sync;
use rocket::futures::{SinkExt, StreamExt};
use rocket_ws::{Message, result, stream::DuplexStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use crate::types::Room;
use crate::types::{MsgBroadcastReceiver, MsgBroadcastSender};

type WsRes = Result<bool, result::Error>;

pub async fn ws_step(
    rx: &mut MsgBroadcastReceiver,
    tx: &MsgBroadcastSender,
    stream: &mut DuplexStream,
    room: &mut Sync<Room>,
    id: Uuid,
) -> WsRes {
    tokio::select! {
        Some(res) = stream.next() => {
            match res {
                Err(err) => {
                    stream_handle_err(tx, room, stream, &id).await?;

                    Err(err)
                }
                Ok(msg) => {
                    match msg {
                        Message::Text(txt) => text_branch(tx, room, stream, &id, &txt).await,
                        Message::Close(_) => {
                            if let Some(user) = room.lock().await.get_user(id) {
                                let _ = tx.send(ServerMessage::UserLeft(user.clone()));
                            }
                            Ok(true)
                        },
                        _ => Ok(true)
                    }
                }
            }
        }
        res = rx.recv() => rx_branch(tx, room, stream, &id, res).await,
        else => {
            if let Some(user) = room.lock().await.get_user(id) {
                let _ = tx.send(ServerMessage::UserLeft(user.clone()));
            }
            Ok(true)
        }
    }
}

async fn text_branch(
    tx: &MsgBroadcastSender,
    room: &mut Sync<Room>,
    stream: &mut DuplexStream,
    id: &Uuid,
    txt: &str,
) -> WsRes {
    if let Ok(msg) = serde_json::from_str::<ClientMessage>(txt) {
        match msg {
            ClientMessage::SendMessage(message) => {
                let _ = tx.send(ServerMessage::NewMessage(ChatMessage::new(*id, message)));
            }
            ClientMessage::ChangeUserName(name) => {
                let mut room = room.lock().await;
                if room.has_user(*id) {
                    let user = room.users.iter_mut().find(|u| *u.get_id() == *id).unwrap();
                    user.set_name(name);
                    let _ = tx.send(ServerMessage::UserNameChange(user.clone()));
                } else {
                    stream
                        .send(ServerMessage::InvalidUser(*id).as_wsmsg())
                        .await?;
                }
            }
            ClientMessage::GetUserData(uuid) => {
                if let Some(user) = room.lock().await.get_user(uuid) {
                    stream
                        .send(ServerMessage::UserData(user.clone()).as_wsmsg())
                        .await?;
                } else {
                    stream
                        .send(ServerMessage::InvalidUser(uuid).as_wsmsg())
                        .await?;
                }
            }
            ClientMessage::GetSelf => {
                let room = room.lock().await;
                let user = room.users.iter().find(|u| *u.get_id() == *id).unwrap();
                stream
                    .send(ServerMessage::SelfData(user.clone()).as_wsmsg())
                    .await?;
            }
        }
    } else {
        let _ = tx.send(ServerMessage::NewMessage(ChatMessage::new(
            *id,
            txt.to_string(),
        )));
    }
    Ok(false)
}

async fn rx_branch(
    tx: &MsgBroadcastSender,
    room: &mut Sync<Room>,
    stream: &mut DuplexStream,
    id: &Uuid,
    res: Result<ServerMessage, RecvError>,
) -> WsRes {
    match res {
        Ok(msg) => {
            stream.send(msg.as_wsmsg()).await?;
            log::info!("User sent: {:?}", msg);
            Ok(false)
        }
        Err(broadcast::error::RecvError::Closed) => {
            if let Some(user) = room.lock().await.get_user(*id) {
                let _ = tx.send(ServerMessage::UserLeft(user.clone()));
            }
            Ok(true)
        }
        Err(broadcast::error::RecvError::Lagged(n)) => {
            log::warn!("Broadcast lagged behind by {n} messages");
            Ok(false)
        }
    }
}

async fn stream_handle_err(
    tx: &MsgBroadcastSender,
    room: &mut Sync<Room>,
    stream: &mut DuplexStream,
    id: &Uuid,
) -> Result<(), result::Error> {
    let mut room = room.lock().await;
    let user = room.get_user(*id);
    stream.close(Default::default()).await?;
    if let Some(user) = user {
        let _ = tx.send(ServerMessage::UserLeft(user.clone()));
    }
    room.remove_user(*id);

    Ok(())
}
