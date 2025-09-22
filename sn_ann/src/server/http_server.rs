use crate::{
    ann::AnnIndex,
    error::{ErrorAnn, Result},
    server::{app_state::AppState, embedding, partition, ping},
};
use http::StatusCode;
use sn_core::server::defauft_config::{
    DEFAULT_SERVER_ANN_HOST, DEFAULT_SERVER_ANN_PORT, DEFAULT_SERVER_ANN_PROTOCOL,
};
use std::{env, sync::Arc};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::{error, info, Level};

/// Simple fallback handler for unmatched routes.
async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

#[tokio::main]
pub async fn http_server_ann(ann_idx: AnnIndex) -> Result<()> {
    let host = env::var("SERVER_ANN_HOST").unwrap_or(String::from(DEFAULT_SERVER_ANN_HOST));
    let port = env::var("SERVER_ANN_PORT").unwrap_or(String::from(DEFAULT_SERVER_ANN_PORT));
    let protocol =
        env::var("SERVER_ANN_PROTOCOL").unwrap_or(String::from(DEFAULT_SERVER_ANN_PROTOCOL));
    // Initialize shared application state
    let app_state = Arc::new(AppState::new(ann_idx));
    let routes_api = axum::Router::new()
        .merge(embedding::route::routes())
        .merge(partition::route::routes())
        .merge(ping::route::routes())
        .with_state(app_state.clone());

    // Build versioned API routes
    let router = axum::Router::new()
        .nest("/api", routes_api)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
        .fallback(fallback);

    let listener = match tokio::net::TcpListener::bind(format!("{host}:{port}")).await {
        Ok(listener) => {
            info!("Starting HTTP server on {protocol}://{host}:{port}");
            listener
        }
        Err(err) => {
            error!("Failed to bind to {host}:{port}. {}", err);
            return Err(ErrorAnn::from(err));
        }
    };
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
