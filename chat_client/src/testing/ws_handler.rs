use chat_lib::ws_connection::Message;
use std::sync::mpsc::{Receiver as SyncReceiver, SyncSender, sync_channel};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::ws_handler::{WsAction, WsEvent};

#[allow(unused)]
pub struct HandlerChannels {
    // event channels
    pub e_tx: Sender<WsEvent>,
    pub e_rx: Receiver<WsEvent>,
    // actions channels
    pub a_tx: SyncSender<WsAction>,
    pub a_rx: SyncReceiver<WsAction>,

    // in this situation `in` is the messages the server would send
    // and `out` is what the server would recieve
    pub in_tx: Sender<Message>,
    pub in_rx: Receiver<Message>,
    pub out_tx: Sender<Message>,
    pub out_rx: Receiver<Message>,
}

impl HandlerChannels {
    pub fn new() -> Self {
        let (e_tx, e_rx) = channel(32);
        let (a_tx, a_rx) = sync_channel(32);

        // in this situation `in` is the messages the server would send
        // and `out` is what the server would recieve
        let (in_tx, in_rx) = channel(32);
        let (out_tx, out_rx) = channel(32);

        Self {
            e_tx,
            e_rx,
            a_tx,
            a_rx,
            in_tx,
            in_rx,
            out_tx,
            out_rx,
        }
    }
}
