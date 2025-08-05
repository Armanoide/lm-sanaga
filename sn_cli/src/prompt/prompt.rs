use crate::client::CliClient;
use crate::error::Result;
use std::io;
use std::io::{BufRead, Write};
use std::sync::Arc;
use serde::Deserialize;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::types::stream_data::StreamData;
use crate::prompt::session::prompt_session;
use crate::utils::stream_response_bytes::{stream_response_bytes};
use crate::utils::typewriter::typewriter;

#[derive(Debug, Deserialize, Default)]
struct Metadata {
    pub message_id: Option<i32>,
    pub generation_tps: Option<f64>,
    pub prompt_tps: Option<f64>,
}
#[derive(Debug, Default)]
struct ResponseInfo {
    pub has_error: bool,
    pub metadata: Metadata,
}

fn handle_response_stream_data(stream_data: &StreamData, response_info: &mut ResponseInfo) {
    if !stream_data.error.is_empty() {
        response_info.has_error = true;
        eprintln!("[ERROR]: {}", stream_data.error);
    }

    if !stream_data.content.is_empty() {
        typewriter(&stream_data.content, 5);
    }

    if !stream_data.metadata.is_null() {
        if let Ok(metadata) = serde_json::from_value::<Metadata>(stream_data.metadata.clone()) {
            response_info.metadata = metadata;
        }
    }
}

pub async fn simple_prompt(cli_client: &CliClient, model_id: Arc<str>) -> Result<()> {
    println!("Model launched in container {}\n", model_id);
    let session_id = prompt_session(&cli_client).await?;
    println!("Starting simple prompt. Type 'exit' or 'quit' to exit.\n");
    let stdin = io::stdin();
    let mut last_response_info = ResponseInfo::default();

    loop {
        print!("> ");
        // Flush stdout to ensure the prompt is visible
        io::stdout().flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            println!("Failed to read input. Try again.");
            continue;
        }

        let prompt = input.trim().to_string();
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Exiting prompt.");
            break;
        }

        let response = cli_client.send_prompt(&GenerateTextRequest {
            model_id: model_id.clone(),
            prompt,
            stream: true,
            last_message_id: last_response_info.metadata.message_id,
            session_id,
        }).await?;

        let mut stream = stream_response_bytes(response.bytes_stream()).await;

        while let Some(line) = stream.recv().await {
            if let Some(data) = line.strip_prefix("data: ") {
                let result_parse = serde_json::from_str::<StreamData>(&data);
                let stream_data: StreamData = match result_parse {
                    Ok(s) => s,
                    Err(err) =>  {
                        println!("[ERROR]: Failed to parse stream data: {} with ({})", err, line);
                        continue;
                    },
                };
                handle_response_stream_data(&stream_data, &mut last_response_info);
            }
        }
        println!(
            "\n====\nPrompt TPS: {:>8.2} tokens/sec\nGen    TPS: {:>8.2} tokens/sec",
            last_response_info
                .metadata
                .prompt_tps
                .unwrap_or(0.0),
            last_response_info
                .metadata
                .generation_tps
                .unwrap_or(0.0)
        );
        if last_response_info.has_error {
            break;
        }
    }
    Ok(())
}
