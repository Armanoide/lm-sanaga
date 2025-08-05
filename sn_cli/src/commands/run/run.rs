use inquire::{InquireError, Select};
use crate::client::CliClient;
use crate::commands;
use crate::error::Result;
pub async fn handler(cli_client: &CliClient) -> Result<()>{
    let response = cli_client.list_model().await?;
    let models: Vec<String> =
        serde_json::from_str(&response).map_err(|e| sn_core::error::Error::from(e))?;

    let options: Vec<String> = models;

    let ans: std::result::Result<String, InquireError> = Select::new("Choose a model", options).prompt();

    match ans {
        Ok(model) => {
            commands::model::run::handle(cli_client, Some(model)).await?;
        },
        Err(e) => {
            eprintln!("Error selecting model: {}", e);
        }
    }

    Ok(())
}