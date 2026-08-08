#[cfg(feature = "ws_msg")]
pub mod ws_message;

#[cfg(feature = "ws_conn")]
pub mod ws_connection;
#[cfg(feature = "ws_conn")]
pub mod ws_mock;

pub mod consts;
pub mod discovery;
pub mod prelude;
pub mod types;
pub mod version;

pub use discovery::Discovery;
pub use types::{ClientMessage, Message, ServerMessage, User};
pub use version::Version;

#[cfg(feature = "ws_conn")]
pub use ws_connection::WsConnection;

/// Basically a wrapper around [`include_str!`],
/// where the base is `$CARGO_MANIFEST_DIR` instead of cwd
#[macro_export]
macro_rules! text_resource {
    ($file:expr $(,)?) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file))
    };
}
