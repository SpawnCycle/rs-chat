mod app;
mod config;
mod consts;
mod logging;
mod render_parts;
mod room_event;
mod ws_handler;

use anyhow::Result;
use ratatui::crossterm::event;
use std::sync::mpsc::sync_channel;
use tokio::{sync::mpsc::channel, time::timeout};
use tracing::{error, trace};

use app::App;
use consts::{CHANNEL_BUFFER_SIZE, TICK_DURATION};
use ws_handler::{WsAction, WsEvent, WsHandler};

use crate::consts::WS_TIMEOUT_DURATION;

#[tokio::main]
async fn main() -> Result<()> {
    // have to initialize this before logging
    // or it will pollute the logs with the help messages
    let config = config::init();
    let _lhandle = logging::setup();

    let (e_tx, mut e_rx) = channel::<WsEvent>(CHANNEL_BUFFER_SIZE);
    let (a_tx, a_rx) = sync_channel::<WsAction>(CHANNEL_BUFFER_SIZE);

    let mut terminal = ratatui::init();
    let mut app = App::new(a_tx);

    let ws = tokio::spawn(async move {
        let config = config.clone();
        trace!("Websocket handler started");
        let handler = WsHandler::new(e_tx, a_rx, config)
            .await
            .inspect_err(|err| error!("Fatal error during websocket connection: {err}"));
        let Ok(mut handler) = handler else {
            return; // Ok to return because handler is not initialized
        };

        while !handler.step().await {}
        trace!("Websocket handler ended");
    });

    while !app.should_quit() {
        terminal.draw(|f| {
            app.draw(f);
        })?;
        if event::poll(TICK_DURATION)? {
            app.handle_input(event::read().expect("Event read should succeed"));
        }
        app.send_sync_requests();
        while let Ok(action) = e_rx.try_recv() {
            app.handle_event(&action);
        }
    }

    ratatui::restore();

    let _ = timeout(WS_TIMEOUT_DURATION / 2, ws).await.inspect_err(|_| {
        error!("Ws join timed out");
    });

    Ok(())
}
