use crate::error::{Error, ResultAPI};
use crate::utils::parse_json_model_id::parse_json_model_id;
use axum::Json;
use axum::extract::State;
use serde_json::Value;
use serde_json::json;
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use crate::server::app_state::AppState;

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

/// Handler to load and run a model based on its name provided in the JSON payload.
///
/// # Arguments
/// * `State(state)` - Shared application state (provides access to the model runner).
/// * `json` - JSON payload expected to contain a non-empty `"name"` field.
///
/// # Returns
/// * `Ok(Json)` - A JSON response containing the launched model's ID (`{ "id": "<model_id>" }`).
/// * `Err` - If the `"name"` field is missing or empty, or if loading the model fails.
pub async fn run_model(
    State(state): State<Arc<AppState>>,
    json: Json<HashMap<String, Value>>,
) -> ResultAPI {
    if let Some(name) = json
        .get("name")
        .and_then(|name| name.as_str())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
    {
        let model_id = {
            let context = "launching model";
            state.runner.write_lock(context)?.load_model_name(&name)?
        };

        Ok(Json(json!({
            "id": model_id,
        })))
    } else {
        Err(Error::ModelNameRequired)
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
