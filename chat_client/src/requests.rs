use chat_lib::{discovery::Discovery, types::User};
use reqwest::Client;
use url::Url;

/// # Errors
///
/// This function returns the Errors produced by `reqwest` client
///
/// # Panics
///
/// This function panics if the url can't be joined
pub async fn room_ls(client: &Client, url: &Url, room: &str) -> Result<Vec<User>, reqwest::Error> {
    let url = url
        .join(&format!("room/{room}/ls"))
        .expect("The url should be correct");

    log::debug!("Running ls on {url}");

    client.get(url).send().await?.json::<Vec<User>>().await
}

/// # Errors
///
/// This function returns the Errors produced by `reqwest` client
///
/// # Panics
///
/// This function panics if the url can't be joined
pub async fn room_discovery(client: &Client, url: &Url) -> Result<Discovery, reqwest::Error> {
    let url = url.join("about").expect("The url should be correct");

    log::debug!("Discovering {url}");

    client.get(url).send().await?.json::<Discovery>().await
}
