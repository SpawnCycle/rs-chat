use semver::Version;

#[cfg(feature = "server")]
pub mod ws_message;

#[cfg(feature = "ws_conn")]
pub mod ws_connection;
#[cfg(feature = "mock_ws")]
pub mod ws_mock;

pub mod consts;
pub mod discovery;
pub mod prelude;
pub mod types;

pub use discovery::Discovery;
pub use types::{ClientMessage, Message, ServerMessage, User};
pub use ws_connection::WsConnection;

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
