pub(crate) mod args;
pub(crate) mod file;

use clap::Parser;
use std::process;

use self::args::AppArgs;
use self::file::AppConfig;

pub fn init() -> AppConfig {
    let args = AppArgs::parse();
    handle_flags(&args);
    AppConfig::default().merge(args)
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
