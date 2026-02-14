use std::{sync::LazyLock, time::Duration};

use rustrict::{ContextProcessingOptions, ContextRateLimitOptions};

pub static CONTEXT_OPTS: LazyLock<ContextProcessingOptions> =
    LazyLock::<ContextProcessingOptions>::new(|| ContextProcessingOptions {
        block_if_muted: false,
        block_if_empty: false,
        block_if_severely_inappropriate: true,
        rate_limit: Some(ContextRateLimitOptions {
            limit: Duration::from_millis(500),
            burst: 5,
            ..Default::default()
        }),
        trim_whitespace: true,
        ..Default::default()
    });
