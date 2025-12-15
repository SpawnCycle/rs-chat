use tokio_tungstenite::tungstenite::{client::IntoClientRequest, handshake::client::Request};

use super::args::AppArgs;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub url: Request,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // TODO: change it to the url of the pi once this is hosted
            url: "ws://127.0.0.1:8000/ws/"
                .into_client_request()
                .expect("Default connection string should be good"),
        }
    }
}

impl AppConfig {
    pub(super) fn merge(mut self, args: AppArgs) -> Self {
        if let Some(url) = args.url {
            self.url = url.into();
        }

        self
    }
}
