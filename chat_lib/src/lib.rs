use semver::Version;

#[cfg(feature = "ratatui_span")]
pub mod ratatui_span;
#[cfg(feature = "ws_message")]
pub mod ws_message;

pub mod prelude;
pub mod types;

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Rust uses unparsable semver?")
}
