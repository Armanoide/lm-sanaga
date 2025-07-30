use crate::client::CliClient;
use crate::error::Result;

pub async fn handle(cli_client: &CliClient) -> Result<()> {
    let response = cli_client.list_model().await?;
    let models: Vec<String> =
        serde_json::from_str(&response).map_err(|e| sn_core::error::Error::from(e))?;
    match models.len() {
        0 => {
            println!("No models installed found.");
        }
        _ => {
            println!("name\n======");
            for model in models {
                println!("{}", model);
            }
        }
    };
    Ok(())
}
