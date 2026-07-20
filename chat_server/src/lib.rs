use std::time::Duration;

use axum::{
    BoxError, Router,
    error_handling::HandleErrorLayer,
    http::{StatusCode, Uri},
};
use tower::ServiceBuilder;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::ws::SyncRoomComponents;

#[derive(Clone)]
struct AppState {
    components: SyncRoomComponents,
}

mod app_error;
mod config;
mod consts;
pub mod limited_string;
pub mod ws;

pub fn app() -> Router {
    Router::new().merge(ws::paths()).fallback(fallback).layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                    .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
            )
            .layer(HandleErrorLayer::new(|error: BoxError| async move {
                if error.is::<tower::timeout::error::Elapsed>() {
                    Ok(StatusCode::REQUEST_TIMEOUT)
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {error}"),
                    ))
                }
            }))
            .timeout(Duration::from_secs(10))
            .into_inner(),
    )
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("Couldn't find {uri}"))
}
