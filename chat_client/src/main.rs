use anyhow::Context;
use ratatui::crossterm::event;
use tokio::time::timeout;

use chat_client::{
    actions::actions,
    app::App,
    config::{self, AppAction, AppConfig, logging},
    consts::{TICK_DURATION, WS_TIMEOUT_DURATION},
};

fn main() -> anyhow::Result<()> {
    // have to initialize this before logging
    // or it will pollute the logs with the help messages
    let (config, cli) = config::init();
    logging::setup()?;

    // Unwrapping the runtime initialization so clap can exit without messing with it
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Couldn't initialize async runtime")?
        .block_on(async { app_entry_point(config, cli).await })
        .inspect_err(|err| log::error!("{err}"))
}

async fn app_entry_point(config: AppConfig, action: Option<AppAction>) -> anyhow::Result<()> {
    if let Some(action) = action {
        actions(config, action).await?;

        return Ok(());
    }

    let room_name = config.web.default_room.clone();
    let mut app = App::new(config);
    // TODO: handler the error differently in the user action,
    // if and when that will be a thing
    let ws = app.join_room(&room_name).await?;

    let mut terminal = ratatui::init();

    while !app.should_quit() {
        terminal.draw(|f| {
            app.draw(f);
        })?;
        if event::poll(TICK_DURATION)? {
            app.handle_input(event::read().expect("Event read should succeed"));
        }
        app.send_sync_requests();
        app.poll_room_events();
    }

    ratatui::restore();

    let _ = timeout(WS_TIMEOUT_DURATION / 2, ws).await.inspect_err(|_| {
        log::error!("Ws join timed out");
    });

    Ok(())
}
