use crate::client::CliClient;
use crate::error::Result;
use sn_core::dto::model_runtime::ModelRuntimeDTO;

pub async fn handle(cli_client: &CliClient) -> Result<()> {
    let response = cli_client.ps_model().await?;
    let models: Vec<ModelRuntimeDTO> =
        serde_json::from_str(&response).map_err(|e| sn_core::error::Error::from(e))?;
    match models.len() {
        0 => {
            println!("No models running.");
        }
        _ => {
            println!("id name\n======");
            for model in models {
                println!("{} {}", model.id, model.name);
            }
        }
    };
    Ok(())
}
