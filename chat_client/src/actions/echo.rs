use chat_lib::Version;

use crate::{
    config::{AppConfig, EchoArgs},
    consts::WS_TIMEOUT_DURATION,
    helper::connect_room,
};

pub async fn echo_action(config: AppConfig, args: EchoArgs) -> anyhow::Result<()> {
    let mut room = connect_room(
        &config,
        &config.web.url,
        Version::V1,
        &config.web.default_room,
        config.web.defult_name.clone(),
    )
    .await?;

    let text = args.words.join(" ");
    log::debug!("Echoing {text}");

    room.send_text(&text);
    room.quit();

    room.join_task_timeout(WS_TIMEOUT_DURATION).await?;

    Ok(())
}
