use chat_lib::types::Sync;
use chat_lib::types::{ServerMessage, User};

use rocket::{Build, Rocket, State, get, routes};
use rustrict::Context;
use uuid::Uuid;

use crate::{
    loops::WsLoopCtx,
    types::{MsgBroadcastSender, Room},
};

pub fn bind(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/ws", routes![ws_root])
}

#[get("/")]
fn ws_root(
    ws: rocket_ws::WebSocket,
    bc: &State<MsgBroadcastSender>,
    room: &State<Sync<Room>>,
) -> rocket_ws::Channel<'static> {
    let id = Uuid::new_v4();
    let tx = bc.inner().clone();
    let rx = bc.subscribe();
    let room = room.inner().clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            let new_user = User::new(id, id.to_string());
            {
                room.lock().await.users.push(new_user.to_owned());
            }
            let _ = tx.send(ServerMessage::UserJoined(new_user.to_owned()));

            let ctx = Context::new();
            let mut loop_ctx = WsLoopCtx::new(rx, tx, stream, room, id, ctx);

            loop {
                if loop_ctx.ws_step().await? {
                    break;
                }
            }

            Ok(())
        })
    })
}
