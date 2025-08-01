use crate::client::CliClient;
use crate::error::Result;
use futures_util::StreamExt;
use std::io;
use std::io::{BufRead, Write};
use std::sync::Arc;
use sn_core::server::payload::generate_text_request::GenerateTextRequest;

pub async fn simple_prompt(cli_client: &CliClient, model_id: Arc<str>) -> Result<()> {
    println!(
        "Model launched in container {}\nType your prompt (or 'exit' to quit):",
        model_id
    );

    let mut has_error = false;
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
            last_message_id: None,
            user_id: None,
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
                    if data.starts_with("[ERROR]: Failed to generate text") {
                        has_error = true;
                    }
                    print!("{}", data);
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
