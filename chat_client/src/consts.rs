use std::time::Duration;

/// The buffer size for the various channels (mpsc/broadcast)
pub const CHANNEL_BUFFER_SIZE: usize = 128;

/// The length of the event poll for the tui
pub const TICK_DURATION: Duration = Duration::from_millis(100);

/// The wait time when joining with a ws handler
pub const WS_TIMEOUT_DURATION: Duration = Duration::from_millis(500);

/// The duration for which the action is considered 'pending'
pub const ACTION_LIFETIME: Duration = Duration::from_millis(500);
