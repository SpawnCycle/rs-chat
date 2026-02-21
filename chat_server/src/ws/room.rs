use std::{collections::HashMap, sync::Arc};

use chat_lib::prelude::*;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

use crate::ws::{BroadCastT, MsgBroadcastSender, consts::BROADCAST_BUFFER_SIZE};

pub struct RoomComponents {
    pub room: Arc<Mutex<Room>>,
    pub tx: MsgBroadcastSender,
}

impl Default for RoomComponents {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomComponents {
    #[must_use]
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel::<BroadCastT>(BROADCAST_BUFFER_SIZE);
        Self {
            room: Arc::new(Mutex::new(Room::new())),
            tx,
        }
    }

    #[must_use]
    pub fn sync() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }
}

pub struct Room {
    users: HashMap<Uuid, User>,
}

#[allow(unused)]
impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}

impl Room {
    #[must_use]
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    #[must_use]
    pub fn has_user(&self, id: &Uuid) -> bool {
        self.users.contains_key(id)
    }

    #[must_use]
    pub fn get_user(&self, id: &Uuid) -> Option<&User> {
        self.users.get(id)
    }

    pub fn modify_or_add_user(&mut self, user: User) {
        self.users
            .entry(*user.get_id())
            .and_modify(|usr| *usr = user.clone())
            .or_insert(user);
    }

    pub fn get_user_mut(&mut self, id: &Uuid) -> Option<&mut User> {
        self.users.get_mut(id)
    }

    pub fn remove_user(&mut self, id: &Uuid) -> Option<User> {
        self.users.remove(id)
    }

    pub fn add_user(&mut self, user: User) {
        self.users.entry(*user.get_id()).insert_entry(user);
    }
}
