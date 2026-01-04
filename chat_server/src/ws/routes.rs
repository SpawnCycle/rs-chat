use chat_lib::types::Sync;
use chat_lib::{discovery::Discovery, prelude::*};

use names::{Generator, Name};
use rocket::{Shutdown, State, get, serde::json::Json};
use rustrict::Context;
use uuid::Uuid;

use crate::{
    types::{MsgBroadcastSender, Room},
    ws::handler::WsHandler,
};

#[get("/version")]
pub fn version() -> Json<semver::Version> {
    Json(chat_lib::version())
}

#[get("/discovery")]
pub fn discovery() -> Json<Discovery> {
    Json(Discovery {
        ws: "/ws".to_owned(),
        version: "/version".to_owned(),
    })
}

#[get("/ws")]
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
            let mut loop_ctx = WsHandler::new(stream, ctx, id, rx, tx, room, &mut sd);

            loop {
                if loop_ctx.ws_step().await? {
                    break;
                }
            }

            Ok(())
        })
    })
}
