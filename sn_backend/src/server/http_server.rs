use std::env;
use axum::http::StatusCode;
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};
use axum::response::Response;
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::{error, info, Level};
use tracing::log::warn;
use crate::db::connection::get_connection;
use crate::error::{Error, Result};
use crate::server::app_state::AppState;
use crate::server::{conversation, model, session, text};

/// Default server configuration constants.
const DEFAULT_SERVER_HOST: &str ="127.0.0.1";
const DEFAULT_SERVER_PORT: &str ="3000";
const DEFAULT_SERVER_PROTOCOL: &str = "http";

/// Simple fallback handler for unmatched routes.
async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Starts the HTTP server using Axum and shared Runner and DB state.
///
/// # Arguments
/// * `runner` - A thread-safe reference to the LLM Runner instance shared across routes.
///
/// # Returns
/// * `Result<()>` - Returns `Ok` if the server starts successfully, or an `Error` if it fails.
///
/// # Behavior
/// - Sets up routing for `/api/v1/model`, `/api/v1/text`, `/api/v1/session`
/// - Adds tracing for incoming requests and failures.
/// - Binds to configured host/port and starts listening.
#[tokio::main]
pub async fn http_server(runner: Arc<RwLock<Runner>>) -> Result<()> {
    // Attempt to connect to the database
    let db = get_connection().await.unwrap_or_else(|e| {
        warn!("Failed to connect to the database: {}", e);
        None
    });

    let host = env::var("SERVER_HOST").unwrap_or(String::from(DEFAULT_SERVER_HOST));
    let port = env::var("SERVER_PORT").unwrap_or(String::from(DEFAULT_SERVER_PORT));
    let protocol = env::var("SERVER_PROTOCOL").unwrap_or(String::from(DEFAULT_SERVER_PROTOCOL));
    // Initialize shared application state
    let app_state = Arc::new(AppState::new(runner, db));
    let routes_api = axum::Router::new()
        .merge(model::route::routes())
        .merge(text::route::routes())
        .merge(session::route::routes())
        .merge(conversation::route::routes())
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
        },
        Err(err) => {
            error!("Failed to bind to {host}:{port}. {}", err);
            return Err(Error::from(err));
        }
    };
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
