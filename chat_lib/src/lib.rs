use semver::Version;

#[cfg(feature = "ratatui_span")]
pub mod ratatui_span;
#[cfg(feature = "ws_message")]
pub mod ws_message;

pub mod types;

pub mod prelude;

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Rust uses unparsable semver?")
}
