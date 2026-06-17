use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

use super::args::Cli;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub web: WebConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub url: Url,
    pub default_room: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            web: WebConfig {
                // TODO: change it to the url of the pi once this is hosted
                url: Url::from_str("http://127.0.0.1:8000/api/")
                    .expect("Default Connection url to be correct"),
                default_room: String::from("default"),
            },
        }
    }
}

impl AppConfig {
    #[must_use]
    pub fn merge(mut self, args: &Cli) -> Self {
        if let Some(url) = &args.args.url {
            self.web.url = url.0.clone();
        }

        if let Some(room) = &args.args.room {
            self.web.default_room.clone_from(room);
        }

        self
    }
}
