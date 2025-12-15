use anyhow::anyhow;
use clap::Parser;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct AppArgs {
    /// Sets the connection string, has start with either *ws://* or *wss://*
    #[arg(short, long)]
    pub(super) url: Option<WsUrl>,
}

/// Wrapper type to only parse the ws or wss urls
#[derive(Debug, Clone)]
pub(super) struct WsUrl(pub(super) Url);

impl FromStr for WsUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<WsUrl> {
        let url = Url::from_str(s)?;
        match url.scheme() {
            "ws" | "wss" => Ok(Self(url)),
            _ => Err(anyhow!(
                "The connection string did not specify ws or wss as the protocol"
            )),
        }
    }
}

impl From<WsUrl> for Url {
    fn from(value: WsUrl) -> Self {
        value.0
    }
}
