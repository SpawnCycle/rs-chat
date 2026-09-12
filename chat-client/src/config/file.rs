use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

use super::args::Cli;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub web: WebConfig,
    pub chat: ChatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub url: Url,
    pub default_room: String,
    pub defult_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub buffer_size: usize,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            url: Url::from_str("http://127.0.0.1:8000/")
                .expect("Default Connection url to be correct"),
            default_room: String::from("default"),
            defult_name: None,
        }
    }
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self { buffer_size: 5_000 }
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

        if let Some(name) = &args.args.name {
            self.web.defult_name = Some(name.clone());
        }

        self
    }
}
