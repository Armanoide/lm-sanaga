#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use env_logger::{Builder, Env};
use sn_core::conversation::conversation::{Conversation, Message};
use sn_core::logger::init_tracing;
use sn_inference::runner::Runner;
use std::env;
use std::sync::Arc;
mod app_state;
mod error;
mod middleware;
mod model;
pub mod server;
mod text;
mod utils;

use std::sync::RwLock;
fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    init_tracing();
    let mut runner = Arc::new(RwLock::new(Runner::new()));

    let root_path = "/Volumes/EXT1_SSD/Users/user1/Projects/ML/lm-sanaga/_MODEL/models-llama-3.1-8B-Instruct-4bit".to_owned();

    server::http_server(runner);

    /*let conversation = Conversation::Single(Message {
        //content: String::from("Hi, my name is <name>."),
        content: String::from(
            "i have a dream, that one day this...",
        ),
        role: String::from("user"),
    });

    match (|| {
        runner.load_model_by_path(&root_path)?;
        runner.generate_text("2f566f6c", &conversation)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })() {
        Err(e) => error!("{e}"),
        Ok(_) => (),
    }*/
}
