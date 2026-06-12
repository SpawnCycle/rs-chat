use anyhow::Context;
use chat_client::{
    AppEvent,
    actions::actions,
    app::App,
    config::{self, AppAction, AppConfig, logging},
    consts::CHANNEL_BUFFER_SIZE,
    start_event_poller, start_tick_poller,
};
use tokio::sync::mpsc;

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

    let (tx, mut rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    let mut app = App::new(config);
    app.mock_unimplemented().await?;

    let ev = start_event_poller(tx.clone());
    let tick = start_tick_poller(tx.clone());

    let mut terminal = ratatui::init();

    while !app.should_quit() {
        terminal.draw(|f| {
            app.render(f);
        })?;
        match rx.recv().await {
            Some(AppEvent::Tick) => app.update(),
            Some(AppEvent::Event(ev)) => app.handle_event(&ev),
            Some(AppEvent::Error(err)) => {
                log::error!("There was an error in one of the background tasks: {err}");
                break;
            }
            // Both channels somehow broke, just exit
            None => break,
        }
    }

    ev.abort();
    tick.abort();

    ratatui::restore();

    app.quit();

    Ok(())
}
