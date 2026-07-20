use semver::Version;

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

pub use discovery::Discovery;
pub use types::{ClientMessage, Message, ServerMessage, User};
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

/// Gives back the compiled Version of the lib crate
/// Useful for Syncing the avalable comm versions
/// TODO: Actually implement the versioning
///
/// # Panics
///
/// Panics if it can't parse the semver provided by the crate
#[must_use]
pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Rust uses unparsable semver?")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn crate_version() {
        let _ver = version();
    }
}
