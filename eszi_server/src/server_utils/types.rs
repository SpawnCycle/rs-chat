use tokio::sync::broadcast;

use eszi_lib::messages::types::{ServerMessage, User};
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
        for i in self.users.iter() {
            if *i.get_id() == id {
                return Some(&i);
            }
        }
        None
    }

    pub fn remove_user(&mut self, id: Uuid) {
        self.users.retain(|user| *user.get_id() != id);
    }
}
