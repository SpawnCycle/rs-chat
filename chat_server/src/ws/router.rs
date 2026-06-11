use axum::{Router, routing::get};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::ws::{
    room::RoomComponents,
    routes::{about, room_ls, room_ws},
};

pub fn paths() -> Router {
    let rooms = HashMap::<String, Arc<Mutex<RoomComponents>>>::new();
    let rooms = Arc::new(Mutex::new(rooms));
    let state = crate::AppState { components: rooms };

    Router::new()
        .route("/about", get(about))
        .route("/room/{path}", get(room_ws))
        .route("/room/{path}/ls", get(room_ls))
        .with_state(state)
}
