use reqwest::Client;

use crate::{
    app::App,
    config::{AppConfig, LsArgs},
    ws_handler::{WsAction, WsEvent},
};

// TODO: make ls an endpoint on the server
pub async fn ls_action(config: AppConfig, args: LsArgs) -> anyhow::Result<()> {
    let app = App::new(config);
    let client = Client::new();

    let discovery = app.room_discovery(&client).await?;

    println!("Server version = {}", discovery.version);
    println!("Available rooms = {:?}", discovery.available_rooms);

    if let Some(room) = args.room
        && args.users
    {
        let mut app = app;
        let event = app
            .send_and_wait_for_message_from_room(&room, WsAction::RequestAll, |event| match event {
                WsEvent::UserAllInfo(_) => true,
                _ => false,
            })
            .await?;
        match event {
            Some(WsEvent::UserAllInfo(users)) => {
                let users = users.iter().map(|u| u.get_name()).collect::<Vec<_>>();
                println!("Users in the room = {:?}", users);
            }
            _ => {
                println!("Couldn't get the users in the room");
            }
        }
    }

    Ok(())
}
