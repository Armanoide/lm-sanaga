use crate::client::CliClient;
use crate::error::{Error, Result};

pub async fn handle(cli_client: &CliClient, model_name: Option<String>) -> Result<()> {
    if let Some(name) = model_name {
        let _ = cli_client
            .stop_model(&name)
            .await
            .map_err(|e| Error::FailedToStopModel(name.clone(), e.to_string()))?;
        Ok(())
    } else {
        Err(Error::ModelNotInstalled(String::default()))
    }
}
