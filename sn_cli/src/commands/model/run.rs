use crate::client::CliClient;
use crate::error::{Error, Result};
use crate::prompt::prompt::simple_prompt;
use serde_json::Value;
use std::collections::HashMap;

pub async fn handle(cli_client: &CliClient, model_name: Option<String>) -> Result<()> {
    if let Some(name) = model_name {
        let result = cli_client
            .run_model(&name)
            .await
            .map_err(|e| Error::FailedToRunModel(name.clone(), e.to_string()))?;
        let json_result: HashMap<String, Value> = serde_json::from_str(&result)?;
        if let Some(id) = json_result
            .get("id")
            .and_then(|id| id.as_str())
            .and_then(|id| if id.is_empty() { None } else { Some(id) })
        {
            simple_prompt(cli_client, id).await?;
            Ok(())
        } else {
            Err(Error::UnExpectedRunResponse(name))
        }
    } else {
        Err(Error::ModelNotInstalled(String::default()))
    }
}
