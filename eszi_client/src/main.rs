mod app;
mod consts;
mod helpers;
mod logger;
mod room_event;
mod ws_handler;

use std::io;

use ratatui::crossterm::event;

use app::App;
use std::sync::mpsc::sync_channel;
use tokio::sync::mpsc::channel;

use ws_handler::{WsAction, WsEvent, WsHandler};

use consts::TICK_DURATION;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    logger::setup();

    let (e_tx, mut e_rx) = channel::<WsEvent>(64);
    let (a_tx, a_rx) = sync_channel::<WsAction>(64);

    let mut terminal = ratatui::init();
    let mut app = App::new(a_tx);

    let ws = tokio::spawn(async move {
        log::info!("Websocket handler started");
        let mut handler = WsHandler::new(e_tx, a_rx).await;
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
            app.handle_event(event::read().expect("Could not read event"));
        }
        app.send_sync_requests();
        while let Ok(action) = e_rx.try_recv() {
            app.handle_action(&action);
        }
    }

    let _ = tokio::join!(ws);

    ratatui::restore();

    Ok(())
}
