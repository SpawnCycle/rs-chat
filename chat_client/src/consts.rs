use std::{sync::LazyLock, time::Duration};

use chat_lib::text_resource;
use ratatui::style::Style;
use reqwest::Client;

/// The buffer size for the various channels (mpsc/broadcast)
pub const CHANNEL_BUFFER_SIZE: usize = 128;

/// The length of the event poll for the tui,
/// this affects the shutdown time
pub const POLL_DURATION: Duration = Duration::from_millis(100);

/// A tick event will be sent out after this duration,
/// this affects how responsive the app feels
pub const TICK_DURATION: Duration = Duration::from_millis(100);

/// The wait time when joining with a ws handler
pub const WS_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

/// The duration for which the action is considered 'pending'
pub const ACTION_LIFETIME: Duration = Duration::from_millis(500);

pub const TUI_HELP_TEXT: &str = text_resource!("../const_resources/tui_help.md");

pub static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub const FOCUSED_CURSOR_STYLE: Style = Style::new().reversed().not_underlined();
pub const UNFOCUSED_CURSOR_STYLE: Style = Style::new().not_reversed().underlined();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tui_help_text_exists() {
        assert!(!TUI_HELP_TEXT.is_empty());
    }
}
