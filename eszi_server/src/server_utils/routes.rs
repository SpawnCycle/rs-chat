use eszi_lib::messages::types::{ServerMessage, User};

use rocket::{Build, Rocket, State, get, routes};
use uuid::Uuid;

use super::types::Room;
use eszi_lib::messages::types::Sync;

use crate::server_utils::loops::ws_step;
#[cfg(debug_assertions)]
use crate::server_utils::types::MsgBroadcastSender;

pub fn bind(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/ws", routes![ws_root, notify])
}

#[cfg(debug_assertions)]
#[get("/notify")]
fn notify(bc: &State<MsgBroadcastSender>) {
    let _ = bc.send(ServerMessage::Arbitrary("Halooooooo".into()));
}

#[get("/")]
fn ws_root(
    ws: rocket_ws::WebSocket,
    bc: &State<MsgBroadcastSender>,
    room: &State<Sync<Room>>,
) -> rocket_ws::Channel<'static> {
    let id = Uuid::new_v4();
    let tx = bc.inner().clone();
    let mut rx = bc.subscribe();
    let mut room = room.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let new_user = User::new(id, id.to_string());

            {
                room.lock().await.users.push(new_user.to_owned()); // holy '.'s
            }

            let _ = tx.send(ServerMessage::UserJoined(new_user));

            loop {
                if ws_step(&mut rx, &tx, &mut stream, &mut room, id).await? {
                    break;
                }
            }

            Ok(())
        })
    })
}
