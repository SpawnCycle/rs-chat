pub(crate) mod args;
pub(crate) mod file;

use self::args::AppArgs;
use self::file::AppConfig;
use anyhow::Result;
use clap::Parser;

pub fn init() -> Result<AppConfig> {
    let args = AppArgs::try_parse()?;
    let config = AppConfig::default().merge(args);

    Ok(config)
}
