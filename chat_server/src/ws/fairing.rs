use rocket::{
    Build, Rocket, async_trait,
    fairing::{self, Fairing, Info, Kind},
    routes,
};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

use crate::{
    types::{BroadCastT, Room},
    ws::{
        consts::BROADCAST_BUFFER_SIZE,
        routes::{discovery, version, ws_root},
    },
};

pub struct WsFairing {
    base: String,
}

impl WsFairing {
    #[must_use]
    pub const fn new(base: String) -> Self {
        Self { base }
    }
}

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
        let (tx, _rx) = broadcast::channel::<BroadCastT>(BROADCAST_BUFFER_SIZE);
        let base = self.base.clone();

        log::trace!("Websocket infrastructure initialized");

        Ok(rocket
            .manage(room)
            .manage(tx)
            .mount(base + "/", routes![ws_root, version, discovery]))
    }
}
