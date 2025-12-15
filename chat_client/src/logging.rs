use std::path::PathBuf;

use anyhow::{Context, Result};
use dirs::data_dir;
use log::LevelFilter;
use log4rs::{
    Config, Handle,
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

pub fn setup() -> Result<Handle> {
    let log_path = {
        if cfg!(debug_assertions) {
            PathBuf::from("log/rs_chat_client.log")
        } else {
            data_dir().map_or(PathBuf::from("~/.rs_chat/client.log"), |mut dir| {
                dir.push("rs_chat/client.log");
                dir
            })
        }
    };

    let file_pattern = PatternEncoder::new("{l} - {d(%Y-%m-%d %H:%M:%S)}: {m}{n}");

    let file_logger = FileAppender::builder()
        .encoder(Box::new(file_pattern.clone()))
        .build(log_path)
        .context("File logger filed to initialize")?;

    let config = Config::builder()
        .appender(Appender::builder().build("file_logger", Box::new(file_logger)))
        .build(
            Root::builder()
                .appender("file_logger")
                .build(LevelFilter::Info),
        )
        .context("Config builder failed to build")?;

    let handle = log4rs::init_config(config).context("Config initialization failed")?;

    Ok(handle)
}
