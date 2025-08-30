use std::sync::Arc;
use std::sync::RwLock;

use sn_core::logger::init_tracing;
use sn_inference::ann::TinyAnnStore;
use sn_inference::runner::Runner;

mod db;
mod error;
mod server;
mod utils;

const ANN_STORE_CAPACITY: usize = 1024;
const ANN_STORE_DIM: usize = 16;
const ANN_STORE_PARAM1: usize = 100;
const ANN_STORE_PARAM2: usize = 100;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let runner = Arc::new(RwLock::new(Runner::new()?));
    let ann_store = Arc::new(RwLock::new(TinyAnnStore::new(
        ANN_STORE_CAPACITY,
        ANN_STORE_DIM,
        ANN_STORE_PARAM1,
        ANN_STORE_PARAM2,
    )));

    server::http_server::http_server(runner, ann_store)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
