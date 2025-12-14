use chat_lib::prelude::*;
use chat_lib::types::Sync;

use names::{Generator, Name};
use rocket::{Shutdown, State, get};
use rustrict::Context;
use uuid::Uuid;

use crate::{
    types::{MsgBroadcastSender, Room},
    ws_handler::WsHandler,
};

#[get("/")]
pub fn ws_root(
    ws: rocket_ws::WebSocket,
    bc: &State<MsgBroadcastSender>,
    room: &State<Sync<Room>>,
    sd: Shutdown,
) -> rocket_ws::Channel<'static> {
    let id = Uuid::new_v4();
    let tx = bc.inner().clone();
    let rx = bc.subscribe();
    let room = room.inner().clone();

    ws.channel(move |stream| {
        Box::pin(async move {
            let name = Generator::with_naming(Name::Numbered)
                .next()
                .expect("Generator should not fail");
            let mut sd = sd.clone();
            let new_user = User::new(id, name);
            {
                room.lock().await.add_user(new_user.clone());
            }
            let _ = tx.send(ServerMessage::UserJoined(new_user.clone()));

            let ctx = Context::new();
            let mut loop_ctx = WsHandler::new(rx, tx, stream, room, id, ctx, &mut sd);

            loop {
                if loop_ctx.ws_step().await? {
                    break;
                }
            }

            Ok(())
        })
    })
}
