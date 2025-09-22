use std::sync::{Arc, RwLock};

use sn_core::logger::init_tracing;

use crate::ann::AnnIndex;

const ANN_STORE_DIM: usize = 16;

mod ann;
mod error;
mod server;
mod utils;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let ann_idx = AnnIndex::new(ANN_STORE_DIM);
    server::http_server::http_server_ann(ann_idx);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}
