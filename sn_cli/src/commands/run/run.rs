use inquire::{InquireError, Select};
use crate::client::CliClient;
use crate::commands;
use crate::error::Result;

/// Interactive model selector and runner handler.
///
/// # Description
/// - Fetches the list of models from the backend.
/// - Prompts the user to select a model via terminal UI.
/// - Delegates the selected model to the `run::handle` function.
///
/// # Errors
/// - Returns an error if model listing or parsing fails.
/// - Will also return an error if `run::handle` fails.
pub async fn handler(cli_client: &CliClient) -> Result<()>{
    // Fetch list of models from API
    let response = cli_client.list_model().await?;
    let models: Vec<String> =
        serde_json::from_str(&response).map_err(|e| sn_core::error::Error::from(e))?;

    let options: Vec<String> = models;

    // Prompt the user to select a model
    let ans: std::result::Result<String, InquireError> = Select::new("Choose a model", options).prompt();

    match ans {
        Ok(model) => {
            // Run model selection handler
            commands::model::run::handle(cli_client, Some(model)).await?;
        },
        Err(InquireError::OperationCanceled) => {}
        Err(e) => {
            eprintln!("Error selecting model: {}", e);
        }
    }
    Ok(())
}