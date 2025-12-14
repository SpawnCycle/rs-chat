use std::collections::HashMap;

use tokio::sync::broadcast;

use chat_lib::prelude::*;
use uuid::Uuid;

pub type BroadCastT = ServerMessage;
pub type MsgBroadcastSender = broadcast::Sender<BroadCastT>;
pub type MsgBroadcastReceiver = broadcast::Receiver<BroadCastT>;

pub struct Room {
    users: HashMap<Uuid, User>,
}

#[allow(unused)]
impl Room {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn has_user(&self, id: &Uuid) -> bool {
        self.users.contains_key(id)
    }

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
