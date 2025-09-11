use tokio::sync::broadcast;

use eszi_lib::messages::types::{User, WsMessage};


pub type BroadCastT = WsMessage;
pub type MsgBroadcastSender = broadcast::Sender<BroadCastT>;
pub type MsgBroadcastReceiver = broadcast::Receiver<BroadCastT>;

pub struct Room {
    pub users: Vec<User>,
}

impl Room {
    pub fn new() -> Self {
        Self { users: vec![] }
    }
}
