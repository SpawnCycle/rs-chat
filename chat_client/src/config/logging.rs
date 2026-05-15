use std::path::PathBuf;

use anyhow::{Context, Result};
use dirs::data_dir;
use log::LevelFilter;

#[must_use]
pub const fn log_level() -> LevelFilter {
    if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

/// # Errors
///
/// This function returns an error if the configuration was incorrect
/// or if the `HOME` folder contains a character that's not utf8 encoded
pub fn setup() -> Result<()> {
    // `log4rs` is better, but I couldn't get `tui_logger` to work with it, very sad
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

    tui_logger::init_logger(LevelFilter::Trace)?;
    tui_logger::set_default_level(log_level());
    let tui_file = tui_logger::TuiLoggerFile::new(
        log_path
            .to_str()
            .context("Path name contains non-utf8 character(s)")?,
    );
    tui_logger::set_log_file(tui_file);

    Ok(())
}
