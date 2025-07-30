use crate::client::CliClient;
use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;

pub async fn handle(cli_client: &CliClient) -> Result<()> {
    let response = cli_client.ps_model().await?;
    let models: Vec<HashMap<String, Value>> =
        serde_json::from_str(&response).map_err(|e| sn_core::error::Error::from(e))?;
    match models.len() {
        0 => {
            println!("No models running.");
        }
        _ => {
            println!("id name\n======");
            for model in models {
                let id = model.get("id").and_then(Value::as_str).unwrap_or("unknown");
                let name = model
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                println!("{} {}", id, name);
            }
        }
    };
    Ok(())
}
