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

#[must_use]
pub fn init() -> AppConfig {
    let args = AppArgs::parse();
    handle_flags(&args);

    let config = read_config().unwrap_or_default();
    config.merge(args)
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
fn handle_flags(cfg: &AppArgs) {
    if cfg.default_config {
        let default_cfg =
            toml::to_string_pretty(&AppConfig::default()).expect("Deserializer shouldn't fail");
        println!("{default_cfg}");
        process::exit(0);
    }

    if cfg.clean {
        let log_file = logging::log_path();
        let _ = fs::remove_file(log_file)
            .inspect_err(|err| eprintln!("Couldn't delete log file: {err}"));
    }
}
