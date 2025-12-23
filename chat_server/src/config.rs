use std::{sync::LazyLock, time::Duration};

use rocket::Config;
use rustrict::{ContextProcessingOptions, ContextRateLimitOptions};

// TODO: Actually implement file configs and args
pub fn rocket() -> Config {
    Config {
        cli_colors: false,
        ..Default::default()
    }
}

pub static CONTEXT_OPTS: LazyLock<ContextProcessingOptions> =
    LazyLock::<ContextProcessingOptions>::new(|| ContextProcessingOptions {
        block_if_muted: false,
        block_if_empty: false,
        block_if_severely_inappropriate: false,
        rate_limit: Some(ContextRateLimitOptions {
            limit: Duration::from_millis(500),
            burst: 5,
            ..Default::default()
        }),
        trim_whitespace: true,
        ..Default::default()
    });
