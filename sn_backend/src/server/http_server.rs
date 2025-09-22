use crate::clients::ann::AnnClient;
use crate::error::{ErrorBackend, Result};
use crate::infrastructure::db::connection::get_connection;
use crate::interfaces::{conversation, message, model, session};
use crate::server::app_state::AppState;
use axum::http::StatusCode;
use sn_core::server::defauft_config::{
    DEFAULT_SERVER_BACKEND_HOST, DEFAULT_SERVER_BACKEND_PORT, DEFAULT_SERVER_BACKEND_PROTOCOL,
};
use sn_core::server::routes::print_all_backend_api_paths;
use sn_inference::runner::Runner;
use std::env;
use std::sync::{Arc, RwLock};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::{error, info, Level};
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
/// * `Result<()>` - Returns `Ok` if the server starts successfully, or an `ErrorBackend` if it fails.
///
/// # Behavior
/// - Sets up routing for `/api/v1/model`, `/api/v1/text`, `/api/v1/session`
/// - Adds tracing for incoming requests and failures.
/// - Binds to configured host/port and starts listening.
#[tokio::main]
pub async fn http_server_backend(runner: Arc<RwLock<Runner>>) -> Result<()> {
    // Attempt to connect to the database
    let db = match get_connection().await? {
        Some(db) => db,
        _ => return Err(ErrorBackend::NoDbAvailable),
    };
    let ann = AnnClient::new();
    let host = env::var("SERVER_BACKEND_HOST").unwrap_or(String::from(DEFAULT_SERVER_BACKEND_HOST));
    let port = env::var("SERVER_BACKEND_PORT").unwrap_or(String::from(DEFAULT_SERVER_BACKEND_PORT));
    let protocol = env::var("SERVER_BACKEND_PROTOCOL")
        .unwrap_or(String::from(DEFAULT_SERVER_BACKEND_PROTOCOL));
    // Initialize shared application state
    let app_state = Arc::new(AppState::new(runner, db, ann));
    let routes_api = axum::Router::new()
        .merge(model::route::routes())
        .merge(message::route::routes())
        .merge(session::route::routes())
        .merge(conversation::route::routes())
        .with_state(app_state.clone());

    print_all_backend_api_paths();

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
            return Err(ErrorBackend::from(err));
        }
    };
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
