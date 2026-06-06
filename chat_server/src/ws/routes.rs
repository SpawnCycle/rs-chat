use chat_lib::{discovery::Discovery, prelude::*};

use names::{Generator, Name};
use rocket::{Shutdown, State, get, serde::json::Json};
use rustrict::Context;
use uuid::Uuid;

use crate::ws::{SyncRoomComponents, handler::WsHandler, room::RoomComponents};

#[get("/about")]
pub async fn about(rooms: &State<SyncRoomComponents>) -> Json<Discovery> {
    let rooms = rooms.lock().await.keys().cloned().collect::<Vec<_>>();
    Json(Discovery {
        version: chat_lib::version().to_string(),
        available_rooms: rooms,
    })
}

#[get("/room/<path>/ls")]
pub async fn room_ls(path: &str, rooms: &State<SyncRoomComponents>) -> Json<Vec<User>> {
    let rooms = rooms.inner().clone();
    let rooms = rooms.lock().await;
    let Some(room_components) = rooms.get(path) else {
        return Json(Vec::new());
    };
    let room_components = room_components.lock().await;
    let room = room_components.room.lock().await;

    Json(room.get_all_users())
}

#[get("/room/<path>")]
pub async fn room_ws(
    path: &str,
    ws: rocket_ws::WebSocket,
    rooms: &State<SyncRoomComponents>,
    sd: Shutdown,
) -> rocket_ws::Channel<'static> {
    let path = path.to_string();
    let rooms = rooms.inner().clone();
    let room_components = rooms
        .lock()
        .await
        .entry(path.clone())
        .or_insert(RoomComponents::sync())
        .clone();
    let id = Uuid::new_v4();
    let tx = room_components.lock().await.tx.clone();
    let rx = tx.subscribe();
    let room = room_components.lock().await.room.clone();

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
            let mut loop_ctx = WsHandler::new(stream, ctx, id, rx, tx, room.clone(), &mut sd);

            loop {
                if loop_ctx
                    .ws_step()
                    .await
                    .inspect_err(|err| log::error!("{err}"))
                    .unwrap_or(true)
                {
                    break;
                }
            }
            let _ = loop_ctx
                .cleanup()
                .await
                .inspect_err(|err| log::warn!("Error during cleanup: {err}"));

            if room.lock().await.is_empty() {
                rooms.lock().await.remove_entry(&path);
            }

            Ok(())
        })
    })
}
