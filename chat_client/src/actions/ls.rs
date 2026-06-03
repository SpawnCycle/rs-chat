use reqwest::Client;

use crate::{
    config::{AppConfig, LsArgs},
    requests::{room_discovery, room_ls},
};

pub async fn ls_action(config: AppConfig, args: LsArgs) -> anyhow::Result<()> {
    let client = Client::new();
    let base_url = config.web.url;
    let room_name = config.web.default_room;

    let discovery = room_discovery(&client, &base_url).await?;

    println!("Server version = {}", discovery.version);
    println!("Available rooms = {:?}", discovery.available_rooms);

    if args.users {
        let users = room_ls(&client, &base_url, &room_name).await?;
        let user_names = users
            .into_iter()
            .map(|u| u.get_name().to_string())
            .collect::<Vec<_>>();
        println!("Users in room {room_name} = {user_names:?}");
    }

    Ok(())
}
