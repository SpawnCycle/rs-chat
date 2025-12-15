use anyhow::Context;
use dirs::data_dir;
use log::LevelFilter;
use log4rs::{
    Config,
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use std::path::PathBuf;

pub fn setup() -> Result<log4rs::Handle, anyhow::Error> {
    let log_file = {
        if cfg!(debug_assertions) {
            PathBuf::from("log/rs_chat_server.log")
        } else {
            data_dir().map_or(PathBuf::from("~/.rs_chat/server.log"), |mut dir| {
                dir.push("rs_chat/server.log");
                dir
            })
        }
    };

    let log_level = {
        if cfg!(debug_assertions) {
            LevelFilter::Trace
        } else {
            LevelFilter::Info
        }
    };

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{l} - {d(%Y-%m-%d %H:%M:%S)}: {m}{n}",
        )))
        .build(log_file)
        .context("File logger filed to initialize")?;

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({l})} - {d(%H:%M:%S)}: {m}{n}",
        )))
        .tty_only(true)
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
        .appender(Appender::builder().build("console_appender", Box::new(console_appender)))
        .build(
            Root::builder()
                .appenders(["console_appender", "file_appender"])
                .build(log_level),
        )
        .context("Could not build log config")?;

    let handle = log4rs::init_config(config).context("Logger failed to initialize")?;

    Ok(handle)
}
