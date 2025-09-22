use std::sync::Arc;
use std::sync::RwLock;

use sn_core::logger::init_tracing;
use sn_inference::runner::Runner;

mod application;
mod clients;
mod domain;
mod error;
mod infrastructure;
mod interfaces;
mod server;
mod use_cases;
mod utils;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let runner = Arc::new(RwLock::new(Runner::new()?));
    server::http_server::http_server_backend(runner)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
