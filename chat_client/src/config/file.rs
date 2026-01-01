use std::str::FromStr;
use url::Url;

use super::args::AppArgs;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub url: Url,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // TODO: change it to the url of the pi once this is hosted
            url: Url::from_str("ws://127.0.0.1:8000/chat/ws")
                .expect("Default Connection url to be good"),
        }
    }
}

impl AppConfig {
    pub(super) fn merge(mut self, args: AppArgs) -> Self {
        if let Some(url) = args.url {
            self.url = url.0;
        }

        self
    }
}
