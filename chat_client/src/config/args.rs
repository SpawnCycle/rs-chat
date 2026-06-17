use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use std::str::FromStr;
use url::Url;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(flatten)]
    pub args: GlobalArgs,
    /// Sets the action that needs to be done instead of joining a room
    #[command(subcommand)]
    pub action: Option<ActionType>,
}

#[derive(Debug, Args)]
// #[command(version, about)]
pub struct GlobalArgs {
    /// Sets the base server url all the requests will use
    #[arg(long, global = true)]
    pub url: Option<ServerUrl>,
    /// Sets the room that is joined to by default
    #[arg(short, long, global = true)]
    pub room: Option<String>,
    /// Prints the default config to stdout
    #[arg(long, global = true)]
    pub default_config: bool,
    /// Deletes the log file before starting the client
    #[arg(short, long, global = true)]
    pub clean: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ActionType {
    Ls(LsArgs),
    Echo(EchoArgs),
}

#[derive(Debug, Clone, Args)]
pub struct LsArgs {
    /// Outputs the active users as part of the ls output
    #[arg(short, long)]
    pub users: bool,
}

#[derive(Debug, Clone, Args)]
pub struct EchoArgs {
    pub words: Vec<String>,
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
            _ => {
                let err =
                    anyhow!("The connection string should be http or https, instead got: {scheme}");
                eprintln!("{err:?}");
                Err(err)
            }
        }
    }
}

impl From<ServerUrl> for Url {
    fn from(value: ServerUrl) -> Self {
        value.0
    }
}
