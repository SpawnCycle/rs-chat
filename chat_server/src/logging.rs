use anyhow::Context;
use dirs::data_dir;
use log::LevelFilter;
use log4rs::{
    Config,
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
};
use std::path::PathBuf;

pub fn setup() -> Result<log4rs::Handle, anyhow::Error> {
    let log_file = {
        if cfg!(debug_assertions) {
            PathBuf::from("log/rs_chat_server.log")
        } else {
            data_dir()
                .map(|mut dir| {
                    dir.push("/rs_chat/server.log");
                    dir
                })
                .unwrap_or(PathBuf::from("~/.rs_chat/server.log"))
        }
    };

    let pattern = PatternEncoder::new("{l} - {d(%Y-%m-%d %H:%M:%S)}: {m}{n}");

    let (file_level, console_level) = {
        if cfg!(debug_assertions) {
            // output more to console because files can inflate very fast from excessive loggin
            (LevelFilter::Debug, LevelFilter::Trace)
        } else {
            // output more to file to have more context for reports
            (LevelFilter::Info, LevelFilter::Warn)
        }
    };

    let file_appender = FileAppender::builder()
        .encoder(Box::new(pattern.clone()))
        .build(log_file)
        .context("File logger filed to initialize: {}")?;

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(pattern.clone()))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
        .appender(Appender::builder().build("console_appender", Box::new(console_appender)))
        .logger(
            Logger::builder()
                .additive(true)
                .appender("console_appender")
                .build("app::console_logger", console_level),
        )
        .build(Root::builder().appender("file_appender").build(file_level))
        .context("Could not build log config: {}")?;

    let handle = log4rs::init_config(config).context("Logger failed to initialize: {}")?;

    Ok(handle)
}
