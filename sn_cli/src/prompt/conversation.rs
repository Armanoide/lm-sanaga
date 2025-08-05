use inquire::{InquireError, Select};
use sn_core::types::conversation::Conversation;
use crate::client::CliClient;
use crate::error::Result;

/// Prompts the user to select a conversation from a list of existing conversations.
///
/// # Arguments
/// * `cli_client` - A reference to the CLI client used to fetch conversation data.
/// * `session_id` - An optional session ID to identify which session's conversations to fetch.
///
/// # Returns
/// * `Ok(Some(i32))` - The ID of the selected conversation, if one was selected.
/// * `Ok(None)` - If the session ID was `None` a new conversation will create as anonymous.
/// * `Err` - If an error occurred while fetching or parsing the conversation list.
pub async fn prompt_conversation(cli_client: &CliClient, session_id: Option<&i32>) -> Result<Option<i32>> {
    // If no session ID is provided, return None early
    let session_id = if let Some(session_id) = session_id { session_id }
    else {
        return Ok(None);
    };

    let response = cli_client.list_conversation(session_id).await?;
    let mut conversations: Vec<Conversation> =
        serde_json::from_str(&response).map_err(|e| sn_core::error::Error::from(e))?;

    // Add a placeholder "New Conversation" at the top of the list
    conversations.insert(0, Conversation {
        id: None,
        name: Some(String::from("New Conversation")),
        messages: vec![],
    });

    let options: Vec<Conversation> = conversations;

    let ans: std::result::Result<Conversation, InquireError> = Select::new(
        "Choose a conversation",
        options
    ).prompt();

    // Return the ID of the selected conversation, or None if cancelled
    match ans {
        Ok(conversation) => {
            Ok(conversation.id)
        },
        Err(_) => {
            Ok(None)
        }
    }
}
