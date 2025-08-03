use crate::client::CliClient;
use crate::error::Result;
use futures_util::StreamExt;
use std::io;
use std::io::{BufRead, Write};
use std::sync::Arc;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::types::stream_data::StreamData;
use crate::prompt::session::prompt_session;

fn typewriter(text: &str, delay_ms: u64) {
    let delay = std::time::Duration::from_millis(delay_ms);
    for c in text.chars() {
        print!("{c}");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(delay);
    }
}

pub async fn simple_prompt(cli_client: &CliClient, model_id: Arc<str>) -> Result<()> {
    println!("Model launched in container {}\n", model_id);
    let session_id = prompt_session(&cli_client).await?;
    println!("Starting simple prompt. Type 'exit' or 'quit' to exit.\n");
    let mut has_error = false;
    let mut last_message_id = None;
    let stdin = io::stdin();
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
            last_message_id,
            session_id,
        }).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            for line in chunk.lines() {
                let line = line?;
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "<|eot_id|>" {
                        break;
                    }
                    let stream_data: StreamData = serde_json::from_str(&data)?;

                    if !stream_data.error.is_empty() {
                        has_error = true;
                        println!("[ERROR]: {}", stream_data.error);
                    }

                    if !stream_data.content.is_empty() {
                        typewriter(&stream_data.content, 5);
                    }

                    if !stream_data.metadata.is_null() {
                        last_message_id = stream_data.metadata.as_object()
                            .and_then(| obj| obj.get("message_id"))
                            .and_then(| message_id | message_id.as_i64())
                            .and_then(| id | Some(id as i32));
                    }
                }
            }
        }
        println!(); // Print a newline after the response
        if has_error {
            break;
        }
    }
    Ok(())
}
