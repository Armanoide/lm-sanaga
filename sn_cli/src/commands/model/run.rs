use crate::client::CliClient;
use crate::error::{Error, Result};

pub async fn handle(cli_client: &CliClient, model_name: Option<String>) -> Result<()> {
    if let Some(name) = &model_name {
        let t = cli_client.run_model(name).await.map_err(|e| {
            Error::ModelNotInstalled(format!("Model '{}' is not installed: {}", name, e))
        })?;
        println!("Running model {}... {t}", name);
    } else {
        return Err(Error::ModelNotInstalled(String::default()));
    }
    Ok(())
}
