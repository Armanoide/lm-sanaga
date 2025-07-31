use crate::error::{Error, ResultAPI};
use crate::server::AppState;
use axum::Json;
use axum::extract::State;
use serde_json::Value;
use serde_json::json;
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;
use crate::utils::parse_json_model_id::parse_json_model_id;

pub async fn get_model_list(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models_installed: Vec<String> = {
        let context = "reading models installed of the runner";
        let guard = &state.runner;
        guard.read_lock(context)?.scan_model_installed()?
    };
    Ok(Json(models_installed.into()))
}

pub async fn get_models_running(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models = {
        let context = "reading models of the runner";
        let guard = &state.runner;
        &guard.read_lock(context)?.models
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

pub async fn stop_model(
    State(state): State<Arc<AppState>>,
    json: Json<HashMap<String, Value>>,
) -> ResultAPI {
    let id = parse_json_model_id(&json)?;
    let context = "stopping model";
    println!("Stopping model with ID: {}", id);
    state.runner.write_lock(context)?.unload_model(&id);
    Ok(Json(json!({ "status": "stopped" })))
}