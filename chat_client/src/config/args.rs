use std::str::FromStr;

use clap::Parser;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct AppArgs {
    #[arg(short, long)]
    pub(super) url: Option<UrlWrapper>,
}

#[derive(Debug, Clone)]
pub(super) struct UrlWrapper(Request);

impl FromStr for UrlWrapper {
    type Err = tungstenite::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into_client_request()?))
    }
}

impl From<UrlWrapper> for Request {
    fn from(val: UrlWrapper) -> Self {
        val.0
    }
}
