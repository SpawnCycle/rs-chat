use std::{collections::HashMap, sync::Arc};

use chat_lib::types::ServerMessage;
use room::Room;
use tokio::sync::{Mutex, broadcast};

use crate::ws::room::RoomComponents;

mod handler;
pub mod room;
mod router;
mod routes;

pub use router::paths;

pub type BroadCastT = ServerMessage;
pub type MsgBroadcastSender = broadcast::Sender<BroadCastT>;
pub type MsgBroadcastReceiver = broadcast::Receiver<BroadCastT>;

pub type SyncRoomComponents = Arc<Mutex<HashMap<String, Arc<Mutex<RoomComponents>>>>>;
