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
use tokio::sync::mpsc::channel;

use app::App;
use consts::{CHANNEL_BUFFER_SIZE, TICK_DURATION};
use ws_handler::{WsAction, WsEvent, WsHandler};

#[tokio::main]
async fn main() -> Result<()> {
    let _lhandle = logging::setup();

    let config = config::init()?;

    let (e_tx, mut e_rx) = channel::<WsEvent>(CHANNEL_BUFFER_SIZE);
    let (a_tx, a_rx) = sync_channel::<WsAction>(CHANNEL_BUFFER_SIZE);

    let mut terminal = ratatui::init();
    let mut app = App::new(a_tx);

    let ws = tokio::spawn(async move {
        let config = config.clone();
        log::info!("Websocket handler started");
        let handler = WsHandler::new(e_tx, a_rx, config)
            .await
            .inspect_err(|err| log::error!("Fatal error during websocket connection: {err}"));
        let Ok(mut handler) = handler else {
            return;
        };

        while !handler.step().await {}
        log::info!("Websocket handler ended");
    });

    while !app.should_quit() {
        terminal
            .draw(|f| {
                app.draw(f);
            })
            .expect("Could not draw frame");
        if event::poll(TICK_DURATION)? {
            app.handle_input(event::read().expect("Could not read event"));
        }
        app.send_sync_requests();
        while let Ok(action) = e_rx.try_recv() {
            app.handle_event(&action);
        }
    }

    let _ = tokio::join!(ws);

    ratatui::restore();

    Ok(())
}
