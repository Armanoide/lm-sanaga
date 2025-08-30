use crate::error::{ErrorBackend, ResultAPI, ResultAPIStream};
use crate::server::app_state::AppState;
use crate::utils::parse_json_model_id::parse_json_model_id;
use crate::utils::sse_response_builder::SseResponseBuilder;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use crossbeam::channel::{Receiver, Sender, bounded};
use serde_json::Value;
use serde_json::json;
use sn_core::server::payload::run_model_metadata_response_sse::RunModelMetadataResponseSSE;
use sn_core::server::payload::run_model_request::RunModelRequest;
use sn_core::types::stream_data::StreamData;
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

fn handle_error_run_model(err: &String, tx_err: Option<Arc<Sender<StreamData>>>) {
    error!("{}", err);
    let error = format!("Failed to run model: {}", err);
    if let Some(tx_err) = tx_err {
        let _ = tx_err.send(StreamData::for_stream_error(error).into());
    }
}
/// Handler to retrieve the list of installed models.
///
/// # Arguments
/// * `State(state)` - Shared application state containing the model runner.
///
/// # Returns
/// * `Ok(Json)` - A JSON array of installed model names.
/// * `Err` - If acquiring the lock or scanning models fails.
pub async fn get_model_list(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models_installed: Vec<String> = {
        let context = "reading models installed of the runner";
        let guard = &state.runner;
        guard.read_lock(context)?.scan_model_installed()?
    };
    Ok(Json(models_installed.into()))
}

/// Handler to retrieve a list of currently running models.
///
/// # Arguments
/// * `State(state)` - Shared application state (contains the model runner).
///
/// # Returns
/// * `Ok(Json)` - A JSON array of running models, each with `name` and `id`.
/// * `Err` - If a locking or access error occurs.
pub async fn get_models_running(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models = {
        let context = "reading models of the runner";
        let guard = &state.runner;
        &guard.read_lock(context)?.models.read_lock(context)?.clone()
    };
    let models = models
        .iter()
        .map(|model| {
            json!({
                "name": model.name,
                "id": model.id,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(json!(models)))
}

pub async fn run_model_with_sse(
    state: Arc<AppState>,
    payload: Json<RunModelRequest>,
) -> ResultAPIStream {
    let (tx, rx): (Sender<StreamData>, Receiver<StreamData>) = bounded(100);
    let tx = Arc::new(tx);

    let response = SseResponseBuilder::new(rx).build();
    tokio::spawn(async move {
        let model_id = (|| {
            let guard = state.runner.write_lock("launching model")?;
            let model_id = guard.load_model_name(&payload.model_name, Some(tx.clone()))?;
            Ok::<_, ErrorBackend>(model_id)
        })();
        match model_id {
            Ok(model_id) => {
                let _ = tx.send(StreamData::for_metadata_text_generated_sse_response(
                    RunModelMetadataResponseSSE {
                        model_id: Arc::from(model_id.as_str()),
                    },
                ));
            }
            Err(e) => {
                handle_error_run_model(&e.to_string(), Some(tx));
            }
        }
    });
    Ok(response?)
}

pub async fn run_model_with_json(
    state: Arc<AppState>,
    payload: Json<RunModelRequest>,
) -> ResultAPIStream {
    let model_id = {
        let context = "launching model";
        state
            .runner
            .write_lock(context)?
            .load_model_name(&payload.model_name, None)?
    };

    Ok(Json(json!({
        "id": model_id,
    }))
    .into_response())
}

/// Handles the `/run_model` endpoint, allowing clients to run a model either with
/// standard JSON response or using Server-Sent Events (SSE) for streaming.
///
/// # Parameters
/// - `state`: Shared application state wrapped in `Arc`, providing access to the model runner.
/// - `payload`: JSON request payload parsed into `RunModelRequest`. If the payload is invalid,
///   a `ModelNameRequired` error is returned.
///
/// # Behavior
/// - If `stream` is `true` in the payload, the model will be run using SSE and a streaming
///   response will be returned.
/// - If `stream` is `false` or not provided, the function returns a standard JSON response
///   containing the model ID.
///
/// # Returns
/// - `ResultAPIStream`: Either a streaming SSE response or a JSON response with the model ID.
///   ErrorBackends are handled and returned using the `ErrorBackend` type.
///
/// # ErrorBackends
/// - Returns `ModelNameRequired` if the request payload is missing or invalid.
/// - Returns appropriate inference or internal errors if model loading fails.
pub async fn run_model(
    State(state): State<Arc<AppState>>,
    payload: std::result::Result<Json<RunModelRequest>, JsonRejection>,
) -> ResultAPIStream {
    let payload = payload.map_err(|_| ErrorBackend::ModelNameRequired)?;

    if payload.stream.unwrap_or(false) {
        Ok(run_model_with_sse(state, payload).await?)
    } else {
        Ok(run_model_with_json(state, payload).await?)
    }
}

/// Handler to stop/unload a running model by its ID.
///
/// # Arguments
/// * `State(state)` - Shared application state (contains the model runner and other resources).
/// * `json` - JSON payload containing the model ID (as a key-value map).
///
/// # Returns
/// * `Ok(Json)` - A JSON response indicating success (`{ "status": "stopped" }`).
/// * `Err` - If the model ID is missing or if locking/unloading fails.
pub async fn stop_model(
    State(state): State<Arc<AppState>>,
    json: Json<HashMap<String, Value>>,
) -> ResultAPI {
    let id = parse_json_model_id(&json)?;
    {
        let context = "stopping model";
        info!("Stopping model with ID: {}", id);
        state.runner.write_lock(context)?.unload_model(&id);
    }
    Ok(Json(json!({ "status": "stopped" })))
}
