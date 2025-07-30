pub(crate) use crate::app_state::AppState;
use crate::routes;
use axum::ServiceExt;
use axum::extract::Path;
use axum::http::{Request, StatusCode};
use axum::response::{Html, Json};
use axum::routing::get;
use axum::serve::Serve;
use sn_core::error::{Error, Result};
use sn_inference::runner::Runner;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, RwLock};

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
#[tokio::main]
pub async fn http_server(runner: Arc<RwLock<Runner>>) -> Result<()> {
    let app_state = Arc::new(AppState::new(runner));

    let routes_api = axum::Router::new()
        .merge(routes::model::routes())
        .with_state(app_state.clone());

    let router = axum::Router::new()
        .nest("/api", routes_api)
        .fallback(fallback);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting HTTP server on {:?}", listener.local_addr()?);
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}
