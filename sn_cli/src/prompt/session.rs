use std::io;
use std::io::{BufRead};
use sn_core::server::payload::create_session_request::CreateSessionRequest;
use sn_core::types::session::Session;
use crate::client::CliClient;
use crate::error::{Result, Error};

pub async fn prompt_session(cli_client: &CliClient) -> Result<Option<i32>> {
    let mut input = String::new();

    println!("Enter session name (or press Enter to use default): ");
    let stdin = io::stdin();
    if let Some(e) = stdin.read_line(&mut input).err() {
        return Err(Error::FailedCreateSession(e.to_string()));
    }
    // check if input is empty
    match cli_client.create_session(&CreateSessionRequest{ name: Some(input) }).await {
        Err(e) => {
            println!("Using session without history: {e}");
            Ok(None)
        },
        Ok(result) => {
            let session: Session = serde_json::from_str(&result)?;
            Ok(Some(session.id))
        }
    }
}