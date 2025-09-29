mod client_utils;

use std::{io, time::Duration};

use ratatui::crossterm::event;

use client_utils::app::App;
use std::sync::mpsc::sync_channel;
use tokio::sync::mpsc::channel;
use uuid::{Uuid, uuid};

use client_utils::ws_handler::{WsAction, WsEvent, WsHandler};

use crate::client_utils::consts::TICK_DURATION;

pub const TEST_UUID: Uuid = uuid!("00000000-0000-0000-0000-ffff00000001");

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("Starting app");

    let (e_tx, mut e_rx) = channel::<WsEvent>(64);
    let (a_tx, a_rx) = sync_channel::<WsAction>(64);

    let mut terminal = ratatui::init();
    let mut app = App::new(a_tx);

    let ws = tokio::spawn(async move {
        let mut handler = WsHandler::new(e_tx, a_rx).await;
        while !handler.step().await {}
        log::info!("Websocket handler Out");
    });

    while !app.should_quit() {
        terminal
            .draw(|f| {
                app.draw(f);
            })
            .expect("Could not draw frame");
        if event::poll(TICK_DURATION)? {
            app.handle_event(event::read().expect("Could not read event"))
                .await;
        }
        while let Ok(action) = e_rx.try_recv() {
            log::info!("Action received");
            match action {
                WsEvent::UserAdd(user) => {
                    log::info!("Action: Add User");
                    app.add_user(user);
                }
                WsEvent::UserRemove(uuid) => {
                    log::info!("Action: Remove User");
                    app.remove_user(uuid);
                }
                WsEvent::UserChange(user) => {
                    log::info!("Action: Change User");
                    app.change_user_name(user);
                }
                WsEvent::Message(message) => {
                    log::info!("Action: Add Message");
                    app.add_message(message);
                }
                WsEvent::Quit => {
                    log::info!("Action: Quit");
                    app.quit()
                }
            }
            log::info!("messages stored: ");
            for msg in &app.messages {
                log::info!("  {}", msg.get_content())
            }
        }
    }

    let _ = tokio::join!(ws);

    ratatui::restore();

    Ok(())
}
