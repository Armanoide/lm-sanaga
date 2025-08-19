use crate::client::CliClient;
use crate::error::Result;
use crate::prompt::conversation::prompt_conversation;
use crate::prompt::session::prompt_session;
use crate::utils::stream_response_bytes::stream_response_bytes;
use crate::utils::typewriter::typewriter;
use inquire::{InquireError, Text};
use serde::Deserialize;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::types::stream_data::{StreamData, StreamDataContent};
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
struct Metadata {
    pub conversation_id: Option<i32>,
    pub generation_tps: Option<f64>,
    pub prompt_tps: Option<f64>,
}
#[derive(Debug, Default)]
struct ResponseInfo {
    pub has_error: bool,
    pub metadata: Metadata,
}

const INFO_QUIT_PROMPT: &str = "Type 'exit' or 'quit' to exit the prompt.";

fn handle_response_stream_data(stream_data: &StreamData, response_info: &mut ResponseInfo) {
    if !stream_data.error.is_empty() {
        response_info.has_error = true;
        eprintln!("[ERROR]: {}", stream_data.error);
    }

    match &stream_data.content {
        StreamDataContent::String(content) => {
            typewriter(&content, 5);
        }
        StreamDataContent::TextGeneratedMetadataResponseSSE(content) => {
            response_info.metadata.conversation_id = Some(content.conversation_id);
            response_info.metadata.generation_tps = content.generation_tps;
            response_info.metadata.prompt_tps = content.prompt_tps;
        }
        _ => {}
    }
}

/// Interactively prompts the user for input, sends it to a model for text generation,
/// and displays streamed responses in real-time.
///
/// This function:
/// - Prompts the user to select or create a session and conversation.
/// - Continuously asks for user input until "exit" or "quit" is typed.
/// - Sends each prompt to the specified model using the `CliClient`.
/// - Streams and prints the response as it arrives.
/// - Handles prompt and generation performance (TPS) statistics.
///
/// # Arguments
/// * `cli_client` - A reference to the CLI client used for communication.
/// * `model_id` - The ID of the model to use, wrapped in an `Arc<str>`.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` on success or an error if something fails.
pub async fn simple_prompt(cli_client: &CliClient, model_id: Arc<str>) -> Result<()> {
    let session_id = prompt_session(&cli_client).await?;
    let conversation_id = prompt_conversation(&cli_client, session_id.as_ref()).await?;
    let mut last_response_info = ResponseInfo::default();
    last_response_info.metadata.conversation_id = conversation_id;
    println!("Model launched in container {}", model_id);
    println!("{}\n", INFO_QUIT_PROMPT);

    loop {
        let name = Text::new("").prompt();
        let input = match name {
            Ok(name) => name,
            Err(InquireError::OperationCanceled) => {
                continue;
            }
            Err(InquireError::OperationInterrupted) => {
                println!("{}", INFO_QUIT_PROMPT); // User pressed Ctrl+C
                continue;
            }
            Err(_) => {
                println!("Failed to read input. Try again.");
                continue;
            }
        };

        let prompt = input.trim().to_string();
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Exiting prompt.");
            break;
        }

        let response = cli_client
            .send_prompt(&GenerateTextRequest {
                model_id: model_id.clone(),
                prompt,
                stream: Some(true),
                conversation_id: last_response_info.metadata.conversation_id,
                session_id,
            })
            .await?;

        let mut stream = stream_response_bytes(response.bytes_stream()).await;

        while let Some(line) = stream.recv().await {
            if let Some(data) = line.strip_prefix("data: ") {
                let result_parse = serde_json::from_str::<StreamData>(&data);
                let stream_data: StreamData = match result_parse {
                    Ok(s) => s,
                    Err(err) => {
                        println!(
                            "[ERROR]: Failed to parse stream data: {} with ({})",
                            err, line
                        );
                        continue;
                    }
                };
                handle_response_stream_data(&stream_data, &mut last_response_info);
            }
        }
        println!(
            "\n====\nPrompt TPS: {:>8.2} tokens/sec\nGen    TPS: {:>8.2} tokens/sec",
            last_response_info.metadata.prompt_tps.unwrap_or(0.0),
            last_response_info.metadata.generation_tps.unwrap_or(0.0)
        );

        // Stop the loop if an error occurred during stream processing
        if last_response_info.has_error {
            break;
        }
    }
    Ok(())
}
