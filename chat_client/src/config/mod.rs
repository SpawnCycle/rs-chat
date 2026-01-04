pub(crate) mod args;
pub(crate) mod file;

use clap::Parser;
use dirs::config_dir;
use std::path::PathBuf;
use std::{fs, process};

use self::args::AppArgs;
use self::file::AppConfig;

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
                .inspect_err(|err| eprintln!("Error during config parse: {err}"))
                .ok()
        })
        .ok()
        .flatten()
}

/// Just like clap, exits if it need to do some trivial thing and nothing else
fn handle_flags(cfg: &AppArgs) {
    if cfg.default_config {
        let default_cfg =
            toml::to_string_pretty(&AppConfig::default()).expect("Deserializer shouldn't fail");
        print!("{default_cfg}");
        process::exit(0);
    }
}
