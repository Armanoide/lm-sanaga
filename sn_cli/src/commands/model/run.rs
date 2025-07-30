use crate::client::CliClient;
use crate::error::{Error, Result};
use sn_core::dto::model_runtime::ModelRuntimeDTO;

pub async fn handle(cli_client: &CliClient, model_name: Option<String>) -> Result<()> {
    if let Some(name) = &model_name {
        cli_client.run_model(name).await.map_err(|e| {
            Error::ModelNotInstalled(format!("Model '{}' is not installed: {}", name, e))
        })?;
        println!("Running model {}...", name);
    } else {
        return Err(Error::ModelNotInstalled(String::default()));
    }
    Ok(())
    /*let response = cli_client.get_models().await?;
    let models: Vec<ModelRuntimeDTO> = serde_json::from_str(&response)
        .map_err(|e| sn_core::error::Error::from(e))?;
    match models.len() {
        0 => {
            println!("No models found.");
        },
        _ => {
            println!("id name\n======");
            for model in models {
                println!("{} {}", model.id, model.name);
            }
        }
    };
    Ok(())*/
}
