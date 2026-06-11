mod logging;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::setup()?;

    let r = chat_server::app();

    let l = TcpListener::bind("0.0.0.0:8000").await?;

    let addr = l.local_addr()?;

    log::warn!("Listening on {addr}");

    axum::serve(l, r).await?;

    Ok(())
}
