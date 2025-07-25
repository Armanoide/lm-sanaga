mod config;
mod error;
mod utils;
mod runner;
mod factory;
mod quantized;
mod module;
mod conversation;
pub mod tokenizer;
mod model;
mod chat_template;
mod generator;
mod cache;
mod mask;
use std::env;
use env_logger::{Builder, Env};
use log::{error, LevelFilter};
use crate::conversation::{Conversation, Message};
use crate::runner::{Runner};

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
    let root_path = "/Volumes/EXT1_SSD/Users/user1/Projects/ML/sanaga-lm/_MODEL/models-llama-3.1-8B-Instruct-4bit".to_owned();

    let conversation = Conversation::Single(
        Message {
            content: String::from("Hi !"),
            role: String::from("user")
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