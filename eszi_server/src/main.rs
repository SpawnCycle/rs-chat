mod loops;
mod routes;
mod types;

use tokio::sync::broadcast;

use std::sync::Arc;
use tokio::sync::Mutex;

use types::{BroadCastT, Room};

#[rocket::launch]
async fn rocket() -> _ {
    let room = Arc::new(Mutex::new(Room::new()));
    let (tx, _rx) = broadcast::channel::<BroadCastT>(16);

    routes::bind(rocket::build()).manage(tx).manage(room)
}
