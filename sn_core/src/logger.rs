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

    // Allow override via RUST_LOG
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    // Initialize subscriber with pretty formatter (or .json() for structured logs)
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
