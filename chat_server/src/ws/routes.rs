use std::future;

use axum::{
    Json,
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use chat_lib::{discovery::Discovery, prelude::*};

use names::{Generator, Name};
use rustrict::Context;
use uuid::Uuid;

use crate::{
    AppState,
    ws::{handler::WsHandler, room::RoomComponents},
};

/// GET /
pub async fn root() -> &'static str {
    "This route is for cloudflare; Fuck you cloudflare; (clownflare)"
}

/// GET /about
pub async fn about(State(AppState { components: rooms }): State<AppState>) -> Json<Discovery> {
    let rooms = rooms.lock().await.keys().cloned().collect::<Vec<_>>();
    Json(Discovery {
        version: chat_lib::version().to_string(),
        available_rooms: rooms,
    })
}

/// GET /room/{path}/ls
pub async fn room_ls(
    path: Path<String>,
    State(AppState { components: rooms }): State<AppState>,
) -> Json<Vec<User>> {
    let rooms = rooms.lock().await;
    let Some(room_components) = rooms.get(path.as_str()) else {
        return Json(Vec::new());
    };
    let room_components = room_components.lock().await;
    let room = room_components.room.lock().await;

    Json(room.get_all_users())
}

/// GET /room/{path}
pub async fn room_ws(
    ws: WebSocketUpgrade,
    path: Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // TODO: make graceful shutdown
    let sd = future::pending();
    let rooms = state.components;
    let path = path.to_string();
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

    ws.on_upgrade(move |stream| async move {
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
        let mut loop_ctx = WsHandler::new(stream.into(), ctx, id, rx, tx, room.clone(), &mut sd);

        loop {
            if loop_ctx
                .ws_step()
                .await
                .inspect_err(|err| log::info!("Couldn't send a message: {err}"))
                .unwrap_or(true)
            {
                break;
            }
        }

        if room.lock().await.is_empty() {
            rooms.lock().await.remove_entry(&path);
        }
    })
}
