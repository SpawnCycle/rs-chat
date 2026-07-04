use std::time::Duration;

pub const BROADCAST_BUFFER_SIZE: usize = 32;

// Max amount of messages in a `TIMEOUT_WINDOW`
pub const MESSAGE_LIMIT: usize = 64;

pub const TIMEOUT_WINDOW: Duration = Duration::from_secs(2);

pub const TIMEOUT_DURATION: Duration = Duration::from_secs(10);
