mod ws_handler;

pub use ws_handler::*;

use tokio::sync::mpsc::Receiver;

pub fn rx_collect_available<T>(rx: &mut Receiver<T>) -> Vec<T> {
    let mut out = Vec::new();

    while let Ok(msg) = rx.try_recv() {
        out.push(msg);
    }

    out
}
