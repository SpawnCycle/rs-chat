use chat_lib::{WsConnection, ws_connection::Message, ws_mock::MockWebSocket};
use std::sync::mpsc::{Receiver as SyncReceiver, SyncSender, sync_channel};
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::{
    config::WebConfig,
    ws_handler::{WsAction, WsEvent, WsHandler},
};

/// runs the handler up to `n` times and returnes the number of loops completed
pub async fn handler_run(handler: &mut WsHandler, n: usize) -> usize {
    let mut i = 0;
    loop {
        i += 1;
        if handler.step().await || i >= n {
            return i;
        }
    }
}

#[allow(unused)]
pub struct HandlerChannels {
    /// event channels
    pub e_tx: Sender<WsEvent>,
    pub e_rx: Receiver<WsEvent>,
    /// action channels
    pub a_tx: SyncSender<WsAction>,
    pub a_rx: SyncReceiver<WsAction>,

    /// in this situation `in` is the messages the server would send
    /// and `out` is what the server would recieve
    pub in_tx: Sender<Message>,
    pub in_rx: Receiver<Message>,
    pub out_tx: Sender<Message>,
    pub out_rx: Receiver<Message>,
}

pub struct ConfiguredHandler {
    pub handler: WsHandler,

    /// event reciever
    pub e_rx: Receiver<WsEvent>,
    /// action sender
    pub a_tx: SyncSender<WsAction>,

    /// `in` is the incoming messages from the network
    /// `out` is the outgoing messages sent to the network
    pub in_tx: Sender<Message>,
    pub out_rx: Receiver<Message>,
}

impl ConfiguredHandler {
    pub fn new() -> Self {
        let HandlerChannels {
            e_tx,
            e_rx,
            a_rx,
            in_tx,
            in_rx,
            out_tx,
            out_rx,
            a_tx,
        } = HandlerChannels::new();

        let conn = WsConnection::Mock(Box::new(MockWebSocket::new_proxy(out_tx, in_rx)));
        let handler = WsHandler::new(WebConfig::default(), conn, e_tx, a_rx);

        Self {
            handler,
            e_rx,
            a_tx,
            in_tx,
            out_rx,
        }
    }
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
