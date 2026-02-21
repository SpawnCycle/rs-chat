use rocket::{
    Build, Rocket, async_trait,
    fairing::{self, Fairing, Info, Kind},
    routes,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, broadcast};

use crate::ws::{
    BroadCastT,
    consts::BROADCAST_BUFFER_SIZE,
    room::RoomComponents,
    routes::{about, room_ws, version},
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
        let rooms = HashMap::<String, Arc<Mutex<RoomComponents>>>::new();
        let rooms = Arc::new(Mutex::new(rooms));
        let (tx, _rx) = broadcast::channel::<BroadCastT>(BROADCAST_BUFFER_SIZE);
        let base = self.base.clone();

        log::trace!("Websocket infrastructure initialized");

        Ok(rocket
            .manage(rooms)
            .manage(tx)
            .mount(base + "/", routes![room_ws, version, about]))
    }
}
