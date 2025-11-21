use tokio::sync::broadcast;

use chat_lib::types::{ServerMessage, User};
use uuid::Uuid;

pub type BroadCastT = ServerMessage;
pub type MsgBroadcastSender = broadcast::Sender<BroadCastT>;
pub type MsgBroadcastReceiver = broadcast::Receiver<BroadCastT>;

pub struct Room {
    pub users: Vec<User>,
}

impl Room {
    pub fn new() -> Self {
        Self { users: vec![] }
    }

    pub fn has_user(&self, id: Uuid) -> bool {
        self.users.iter().any(|other| other.get_id() == &id)
    }

    pub fn get_user(&self, id: Uuid) -> Option<&User> {
        self.users
            .iter()
            .find(|&i| *i.get_id() == id)
            .map(|v| v as _)
    }

    pub fn remove_user(&mut self, id: Uuid) {
        self.users.retain(|user| *user.get_id() != id);
    }
}
