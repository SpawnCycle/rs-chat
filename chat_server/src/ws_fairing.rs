use rocket::{
    Build, Rocket, async_trait,
    fairing::{self, Fairing, Info, Kind},
    routes,
};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

use crate::{
    routes::ws_root,
    types::{BroadCastT, Room},
};

pub struct WsFairing;

#[async_trait]
impl Fairing for WsFairing {
    fn info(&self) -> Info {
        Info {
            name: "Fairing for Ws setup",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let room = Arc::new(Mutex::new(Room::new()));
        let (tx, _rx) = broadcast::channel::<BroadCastT>(16);

        log::trace!("Websocket infrastructure initialized");

        Ok(rocket
            .manage(room)
            .manage(tx)
            .mount("/ws", routes![ws_root]))
    }
}
