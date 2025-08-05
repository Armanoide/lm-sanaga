use crate::cli::{Cli, Commands, ModelCommands};
use crate::client::CliClient;
use crate::error::Result;
use clap::Parser;
mod cli;
mod client;
mod commands;
mod error;
mod prompt;
mod utils;

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }
}

async fn try_main() -> Result<()> {
    let cli = Cli::parse();
    let cli_client = CliClient::new("http://localhost:3000");

    match cli.command {
        Commands::Run { .. } => commands::run::run::handler(&cli_client).await?,
        Commands::Model(model_commands) => match model_commands {
            ModelCommands::List { .. } => commands::model::list::handle(&cli_client).await?,
            ModelCommands::PS { .. } => commands::model::ps::handle(&cli_client).await?,
            ModelCommands::Run { model, .. } => {
                commands::model::run::handle(&cli_client, model).await?
            }
            ModelCommands::Stop { model, .. } => {
                commands::model::stop::handle(&cli_client, model).await?
            }
        },
    }

    Ok(())
}
