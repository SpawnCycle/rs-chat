use better_eszichat::messages::types::{User, WsMessage};

use rocket::{Build, Rocket, State, futures::SinkExt, get, routes};
use rocket_ws::Message;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::types::Room;
use better_eszichat::messages::types::Sync;

use crate::server_utils::loops::ws_loop;
#[cfg(debug_assertions)]
use crate::server_utils::types::MsgBroadcastSender;

pub fn bind(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/ws", routes![ws_root, notify])
}

#[cfg(debug_assertions)]
#[get("/notify")]
fn notify(bc: &State<MsgBroadcastSender>) {
    let _ = bc.send(WsMessage::Arbitrary("Halooooooo".into()));
}

#[get("/")]
fn ws_root(
    ws: rocket_ws::WebSocket,
    bc: &State<MsgBroadcastSender>,
    room: &State<Sync<Room>>,
) -> rocket_ws::Channel<'static> {
    let mut rx = bc.subscribe();
    let tx = bc.inner().clone();
    let room = room.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let id = Uuid::new_v4();
            let new_user = User::new(id, id.to_string());

            {
                room.lock().await.users.push(new_user.to_owned()); // holy '.'s
            }

            let _ = tx.send(WsMessage::UserJoined(new_user));

            loop {
                if ws_loop(id, &mut stream, &mut rx, &tx).await? {
                    break;
                }
            }

            Ok(())
        })
    })
}
