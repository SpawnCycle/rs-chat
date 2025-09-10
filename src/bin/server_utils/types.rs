use tokio::sync::broadcast;

use better_eszichat::messages::types::{User, WsMessage};

use std::sync::Arc;
use tokio::sync::Mutex;

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
