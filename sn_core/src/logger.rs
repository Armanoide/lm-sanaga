use std::env;
use env_logger::{Builder, Env};
use log::LevelFilter;

pub fn init_logger() {
    let sanaga_debug = env::var("SANAGA_DEBUG").unwrap_or_else(|_| "false".to_string());

    let mut builder = Builder::from_env(Env::default().default_filter_or("info"));

    //if sanaga_debug == "true" {
    builder.filter_level(LevelFilter::Debug);
    //} else {
    //builder.filter_level(LevelFilter::Info);
    //}

    builder.init();
}
