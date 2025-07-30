pub(crate) use crate::app_state::AppState;
use crate::model;
use axum::ServiceExt;
use axum::http::StatusCode;
use sn_core::error::Result;
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
#[tokio::main]
pub async fn http_server(runner: Arc<RwLock<Runner>>) -> Result<()> {
    let app_state = Arc::new(AppState::new(runner));

    let routes_api = axum::Router::new()
        .merge(model::route::routes())
        .with_state(app_state.clone());

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting HTTP server on {:?}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
