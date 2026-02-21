use chat_lib::types::ServerMessage;
use room::Room;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, broadcast};

use crate::ws::room::RoomComponents;

mod consts;
mod fairing;
mod handler;
pub mod room;
mod routes;

pub use fairing::WsFairing;

pub type BroadCastT = ServerMessage;
pub type MsgBroadcastSender = broadcast::Sender<BroadCastT>;
pub type MsgBroadcastReceiver = broadcast::Receiver<BroadCastT>;

pub type SyncRoomComponents = Arc<Mutex<HashMap<String, Arc<Mutex<RoomComponents>>>>>;
