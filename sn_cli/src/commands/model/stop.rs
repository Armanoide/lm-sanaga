use crate::client::CliClient;
use crate::error::{ErrorCli, Result};

pub async fn handle(cli_client: &CliClient, model_name: Option<String>) -> Result<()> {
    if let Some(name) = model_name {
        let _ = cli_client
            .stop_model(&name)
            .await
            .map_err(|e| ErrorCli::FailedToStopModel(name.clone(), e.to_string()))?;
        Ok(())
    } else {
        Err(ErrorCli::ModelNotInstalled(String::default()))
    }
}
