pub(crate) mod args;
pub(crate) mod file;

use self::args::AppArgs;
use self::file::AppConfig;
use clap::Parser;

pub fn init() -> AppConfig {
    let args = AppArgs::parse();
    AppConfig::default().merge(args)
}
