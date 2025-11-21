use semver::Version;

pub mod types;

pub fn version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Rust uses unparsable semver?")
}
