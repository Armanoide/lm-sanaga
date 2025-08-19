use crate::client::CliClient;
use crate::error::{Error, Result};
use crate::prompt::prompt::simple_prompt;
use crate::utils::stream_response_bytes::stream_response_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use sn_core::server::payload::run_model_request::RunModelRequest;
use sn_core::types::stream_data::{StreamData, StreamDataContent};

pub async fn handle(cli_client: &CliClient, model_name: Option<String>) -> Result<()> {
    if let Some(model_name) = model_name {
        let mut model_id = None;
        let mut pb: Option<ProgressBar> = None;

        let response = cli_client
            .run_model(&RunModelRequest {
                model_name: model_name.clone(),
                stream: Some(true),
            })
            .await
            .map_err(|e| Error::FailedToRunModel(model_name.clone(), e.to_string()))?;

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
                match stream_data.content {
                    StreamDataContent::RunModelMetadataResponseSSE(data) => {
                        model_id = Some(data.model_id);
                    }
                    StreamDataContent::RunModelResponseSSE(data) => {
                        if pb.is_none() {
                            let progress_bar = ProgressBar::new(data.total_tensors as u64);
                            progress_bar.set_style(
                                ProgressStyle::default_bar()
                                    .template("{prefix} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                                    .unwrap()
                                    .progress_chars("##-"),
                            );
                            progress_bar.set_prefix(data.load_type);
                            pb = Some(progress_bar);
                        }
                        if let Some(pb) = &pb {
                            pb.set_position(data.tensor_index as u64);
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Model run completed");
        }

        if let Some(model_id) = model_id {
            simple_prompt(cli_client, model_id).await?;
            Ok(())
        } else {
            Err(Error::UnExpectedRunResponse(model_name))
        }
    } else {
        Err(Error::ModelNotInstalled(String::default()))
    }
}
