use crate::client::CliClient;
use crate::error::Result;
use std::io;
use std::io::{BufRead, Write};
use std::sync::Arc;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;
use sn_core::types::stream_data::StreamData;
use crate::prompt::session::prompt_session;
use crate::utils::stream_response_bytes::{stream_response_bytes};

fn typewriter(text: &str, delay_ms: u64) {
    let delay = std::time::Duration::from_millis(delay_ms);
    for c in text.chars() {
        print!("{c}");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(delay);
    }
}

fn handle_response_stream_data(stream_data: &StreamData, ) -> (bool, Option<i32>) {
    let mut has_error = false;
    let mut last_message_id = None;
    if !stream_data.error.is_empty() {
        has_error = true;
        eprintln!("[ERROR]: {}", stream_data.error);
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
    (has_error, last_message_id)
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
                let result = handle_response_stream_data(&stream_data);
                has_error = result.0;
                last_message_id = result.1;
            }
        }
        println!(); // Print a newline after the response
        if has_error {
            break;
        }
    }
    Ok(())
}
