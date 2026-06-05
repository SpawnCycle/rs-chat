use tokio::time::timeout;

use crate::{
    app::connect_room,
    config::{AppConfig, EchoArgs},
    consts::WS_TIMEOUT_DURATION,
};

pub async fn echo_action(config: AppConfig, args: EchoArgs) -> anyhow::Result<()> {
    let (mut room, ws) = connect_room(
        config.web.clone(),
        &config.web.url,
        &config.web.default_room,
    )
    .await?;

    let text = args.words.join(" ");
    log::debug!("Echoing {text}");

    room.change_name("echo");
    room.send_text(&text);
    room.quit();

    let _ = timeout(WS_TIMEOUT_DURATION, ws).await;

    Ok(())
}
