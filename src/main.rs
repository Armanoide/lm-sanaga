#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
mod cache;
mod chat_template;
mod config;
mod conversation;
mod error;
mod factory;
mod mask;
mod model;
mod module;
mod quantized;
mod runner;
mod token;
pub mod tokenizer;
mod utils;

use crate::conversation::{Conversation, Message};
use crate::runner::Runner;
use env_logger::{Builder, Env};
use log::{LevelFilter, error};
use std::env;

fn init_logger() {
    let sanaga_debug = env::var("SANAGA_DEBUG").unwrap_or_else(|_| "false".to_string());

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));

    //if sanaga_debug == "true" {
    builder.filter_level(LevelFilter::Debug);
    //} else {
    //builder.filter_level(LevelFilter::Info);
    //}

    builder.init();
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let root_path = "/Volumes/EXT1_SSD/Users/user1/Projects/ML/lm-sanaga/_MODEL/models-llama-3.1-8B-Instruct-4bit".to_owned();

    let conversation = Conversation::Single(Message {
        content: String::from("Hi, my name is <name>."),
        role: String::from("user"),
    });

    init_logger();
    let mut runner = Runner::new();
    match (|| {
        runner.load_model_by_path(&root_path)?;
        runner.generate_text("2f566f6c", &conversation)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })() {
        Err(e) => error!("{e}"),
        Ok(_) => (),
    }
}
