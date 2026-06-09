use reqwest::Client;
use std::sync::mpsc::sync_channel;
use tokio::sync::mpsc::channel;
use url::Url;
use uuid::Uuid;

use crate::{
    config::WebConfig,
    consts::CHANNEL_BUFFER_SIZE,
    requests::room_discovery,
    room::Room,
    ws_handler::{WsAction, WsEvent, WsHandler},
};

/// # Errors
///
/// This function errors if there was a problem during any of the web calls,
/// be it an internet error or a parse one
pub async fn connect_room(
    config: WebConfig,
    base_url: &Url,
    room_name: &str,
) -> anyhow::Result<(Room, tokio::task::JoinHandle<()>)> {
    let (e_tx, e_rx) = channel::<WsEvent>(CHANNEL_BUFFER_SIZE);
    let (a_tx, a_rx) = sync_channel::<WsAction>(CHANNEL_BUFFER_SIZE);

    let client = Client::new();

    let discovery = room_discovery(&client, base_url).await?;

    log::debug!("{base_url} - {discovery:?}");

    let web_config = config.clone();
    let room_string = room_name.to_string();
    let ws = tokio::spawn(async move {
        let handler = WsHandler::new(e_tx, a_rx, web_config, room_string.clone())
            .await
            .inspect_err(|err| log::error!("Fatal error during websocket connection: {err}"));
        log::debug!("Websocket handler for {room_string} started");
        let Ok(mut handler) = handler else {
            return; // Ok to return because handler is not initialized
        };

        while !handler.step().await {}

        handler.close().await;

        log::debug!("Websocket handler for {room_string} ended");
    });

    Ok((Room::new(room_name, a_tx, e_rx), ws))
}

/// returns if the given event satisfies a given action (self id is required for actions related to self)
pub fn event_satisfies_action(ev: &WsEvent, ac: &WsAction, self_id: Option<Uuid>) -> bool {
    match (ev, ac, self_id) {
        (WsEvent::SelfInfo(_), WsAction::RequestSelf, _)
        | (WsEvent::AllUserInfo(_), WsAction::RequestAll, _)
        | (WsEvent::Quit, WsAction::Quit, _) => true,
        (WsEvent::UserInfo(lu), WsAction::RequestUser(ru), _) if lu.get_id() == ru => true,
        (WsEvent::UserChange(u), WsAction::ChangeName(_), Some(self_id))
            if *u.get_id() == self_id =>
        {
            true
        }
        _ => false,
    }
}

/// returns if the event should be buffered
pub fn action_should_buffer(ac: &WsAction) -> bool {
    match ac {
        WsAction::RequestUser(_) | WsAction::RequestAll | WsAction::RequestSelf => true,
        WsAction::ChangeName(_) | WsAction::Message(_) | WsAction::Quit => false,
    }
}
