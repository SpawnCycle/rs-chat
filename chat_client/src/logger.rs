use log::LevelFilter;
use log4rs::{
    Config,
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

pub fn setup() {
    #[cfg(debug_assertions)]
    let log_path = "log/eszi_client.log";
    #[cfg(not(debug_assertions))]
    let log_path = "~/.eszi_chat/client.log";

    let file_logger = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{l} - {d(%Y-%m-%d %H:%M:%S)}: {m}{n}",
        )))
        .build(log_path)
        .expect("File log builder failed");

    let config = Config::builder()
        .appender(Appender::builder().build("file_logger", Box::new(file_logger)))
        .build(
            Root::builder()
                .appender("file_logger")
                .build(LevelFilter::Info),
        )
        .expect("Log config builder failed");

    let _handle = log4rs::init_config(config).expect("Log initialization failed");
}
