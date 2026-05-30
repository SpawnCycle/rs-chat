#![allow(
    clippy::print_stderr,
    clippy::print_stdout,
    reason = "This runs before the logger is initialized"
)]

pub(crate) mod args;
pub(crate) mod file;
pub mod logging;

pub use args::*;
pub use file::*;

use clap::Parser;
use dirs::config_dir;
use std::{fs, path::PathBuf, process};

/// Returns the app config as well as the app action (if any)
#[must_use]
pub fn init() -> (AppConfig, Option<AppAction>) {
    let cli = Cli::parse();
    handle_flags(&cli);

    let config = read_config().unwrap_or_default();
    let config = config.merge(&cli);

    (config, cli.action)
}

fn read_config() -> Option<AppConfig> {
    let config_path = config_dir().map_or(PathBuf::from("~/.rs_chat/client.toml"), |mut path| {
        path.push("rs_chat/client.toml");
        path
    });

    fs::read_to_string(config_path)
        .map(|s| {
            toml::from_str(&s)
                // log is not initialized at this point so this is fine
                .inspect_err(|err| eprintln!("Couldn't parse config: {err}"))
                .ok()
        })
        .ok()
        .flatten()
}

/// Executes the trivial things and possibly exits
fn handle_flags(cfg: &Cli) {
    if cfg.args.default_config {
        let default_cfg =
            toml::to_string_pretty(&AppConfig::default()).expect("Deserializer shouldn't fail");
        println!("{default_cfg}");
        process::exit(0);
    }

    if cfg.args.clean {
        let log_file = logging::log_path();
        let _ = fs::remove_file(log_file)
            .inspect_err(|err| eprintln!("Couldn't delete log file: {err}"));
    }
}
