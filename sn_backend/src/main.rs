use sn_core::logger::init_tracing;
use sn_inference::runner::Runner;
use std::sync::Arc;
mod error;
mod utils;
mod db;
mod server;

use std::sync::RwLock;
fn main() {
    init_tracing();
    let runner = Arc::new(RwLock::new(Runner::new()));
    server::http_server::http_server(runner);
}
