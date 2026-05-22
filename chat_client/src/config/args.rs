use anyhow::anyhow;
use clap::Parser;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct AppArgs {
    /// Sets the base server url all the requests will use
    #[arg(short, long)]
    pub url: Option<ServerUrl>,
    /// Print the default config to stdout
    #[arg(short, long)]
    pub default_config: bool,
    /// Deletes the log file before starting the client
    #[arg(short, long)]
    pub clean: bool,
}

/// Wrapper around Url that checks if it's http(s)
#[derive(Debug, Clone)]
pub struct ServerUrl(pub Url);

impl FromStr for ServerUrl {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<ServerUrl> {
        let url = Url::from_str(s)?;
        let scheme = url.scheme();
        match scheme {
            "http" | "https" => Ok(Self(url)),
            _ => Err(anyhow!(
                "The connection string should be http or https, instead got: {scheme}"
            )),
        }
    }
}

impl From<ServerUrl> for Url {
    fn from(value: ServerUrl) -> Self {
        value.0
    }
}
