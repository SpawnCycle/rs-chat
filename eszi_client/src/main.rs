mod client_utils;

use std::{io, time::Duration};

use ratatui::crossterm::event;

use client_utils::app::App;
use tokio::sync::mpsc;
use uuid::{Uuid, uuid};

use client_utils::ws_handler::{WsAction, WsHandler};

pub const TEST_UUID: Uuid = uuid!("00000000-0000-0000-0000-ffff00000001");

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    log::info!("Starting app");

    let mut terminal = ratatui::init();
    let mut app = App::new();

    let (tx, mut rx) = mpsc::channel::<WsAction>(64);

    tokio::spawn(async move {
        let mut handler = WsHandler::new(tx).await;
        while !handler.step().await {}
        log::info!("Websocket handler Out");
    });

    while !app.should_quit() {
        terminal
            .draw(|f| {
                app.draw(f);
            })
            .expect("Could not draw frame");
        if event::poll(Duration::from_millis(25))? {
            app.handle_event(event::read().expect("Could not read event"))
                .await;
        }
        while let Ok(action) = rx.try_recv() {
            log::info!("Action received");
            match action {
                WsAction::UserAdd(user) => {
                    log::info!("Action: Add User");
                    app.add_user(user);
                }
                WsAction::UserRemove(uuid) => {
                    log::info!("Action: Remove User");
                    app.remove_user(uuid);
                }
                WsAction::UserChange(user) => {
                    log::info!("Action: Change User");
                    app.change_user_name(user);
                }
                WsAction::Message(message) => {
                    log::info!("Action: Add Message");
                    app.add_message(message);
                }
                WsAction::Quit => {
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

    ratatui::restore();

    Ok(())
}
