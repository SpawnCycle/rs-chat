use anyhow::Context;
use tokio::sync::mpsc;

use chat_client::{
    actions,
    app::{App, ExitReason},
    config::{self, ActionType, AppConfig, logging},
    consts::CHANNEL_BUFFER_SIZE,
    start_event_poller, start_tick_poller,
};

fn main() -> anyhow::Result<()> {
    // We have to initialize this before logging
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

async fn app_entry_point(config: AppConfig, action: Option<ActionType>) -> anyhow::Result<()> {
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
        if let Err(err) = terminal.draw(|f| {
            app.render(f);
        }) {
            app.exit_because(err.into());
        }
        if let Some(ev) = rx.recv().await {
            app.handle_event(ev).await;
        } else {
            app.exit_because(anyhow::anyhow!("Event channel broke"));
            break;
        }
    }

    ev.abort();
    tick.abort();

    ratatui::restore();

    app.quit();

    #[allow(
        clippy::print_stderr,
        reason = "we're reporing an error with the terminal in a restored state"
    )]
    if let Some(reason) = app.exit_reason() {
        match reason {
            ExitReason::BackgroundError(err) => {
                eprintln!("Exited because background process returned: {err}");
            }
            ExitReason::FatalError(err) => {
                eprintln!("Exited because of a fatal error: {err}");
            }
            ExitReason::UserAction => {
                // No reason to report an exit initialized by the user
            }
        }
    } else {
        eprintln!("Why did we exit?");
    }

    Ok(())
}
