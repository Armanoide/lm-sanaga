use sn_core::logger::init_tracing;
use sn_inference::runner::Runner;
use std::sync::Arc;
mod db;
mod error;
mod server;
mod utils;

use std::sync::RwLock;
fn main() {
    init_tracing();
    let runner = Arc::new(RwLock::new(Runner::new()));
    server::http_server::http_server(runner);
}
