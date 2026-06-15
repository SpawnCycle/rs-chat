use std::time::Duration;

use chat_lib::text_resource;

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
