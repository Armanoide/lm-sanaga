use std::env;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    // Check for SANAGA_DEBUG environment variable
    let sanaga_debug = env::var("SANAGA_DEBUG").unwrap_or_else(|_| "false".to_string());
    // Set base filter depending on debug flag
    let default_level = if sanaga_debug == "true" {
        "debug"
    } else {
        "info"
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}
