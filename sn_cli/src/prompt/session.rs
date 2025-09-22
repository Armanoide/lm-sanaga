use crate::client::CliClient;
use crate::error::{ErrorCli, Result};
use inquire::Text;
use sn_core::server::payload::backend::create_session_request::CreateSessionRequest;
use sn_core::types::session::Session;

pub async fn prompt_session(cli_client: &CliClient) -> Result<Option<i32>> {
    let name = Text::new("Enter session name (or press Enter to use default): ").prompt();

    let name = match name {
        Ok(name) => name,
        Err(e) => {
            return Err(ErrorCli::FailedCreateSession(e.to_string()));
        }
    };

    // check if input is empty
    match cli_client
        .create_session(&CreateSessionRequest { name: Some(name) })
        .await
    {
        Err(e) => {
            println!("Using session without history: {e}");
            Ok(None)
        }
        Ok(result) => {
            let session: Session = serde_json::from_str(&result)?;
            Ok(Some(session.id))
        }
    }
}
