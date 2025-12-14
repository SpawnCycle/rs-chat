use semver::Version;

#[cfg(feature = "ratatui_span")]
pub mod ratatui_span;
#[cfg(feature = "ws_message")]
pub mod ws_message;

pub mod consts;
pub mod prelude;
pub mod types;

/// Gives back the compiled Version of the lib crate
/// Useful for Syncing the avalable comm versions
/// TODO: Actually implement the versioning
///
/// # Panics
///
/// Panics if rust uses bad semver
#[must_use]
pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Rust uses unparsable semver?")
}
