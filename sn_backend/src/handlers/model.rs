use crate::error::{Result, ResultAPI};
use crate::server::AppState;
use axum::Json;
use axum::debug_handler;
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::{Value, json};
use sn_core::dto::model_runtime::ModelRuntimeDTO;
use sn_core::utils::rw_lock::RwLockExt;
use std::collections::HashMap;
use std::sync::Arc;

#[axum::debug_handler]
pub async fn get_model_list(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models_installed = {
        let context = "reading models installed of the runner";
        let guard = &state.runner;
        &guard.read_lock(context)?.scan_model_installed()
    };
    match models_installed {
        Ok(models) => Ok(Json(json!(models))),
        Err(err) => {
            println!("Error scanning models: {}", err);
            let json_error = json!({
                "error": err.to_string(),
                "code": "500"
            });
            Ok(Json(json_error))
            //            let json_error = HashMap::from([
            //                ("error".to_string(), err.to_string()),
            //                ("code".to_string(), "500".to_string()),
            //            ]);
            //            Json(serde_json::to_string(&json_error))
        }
    }
}

pub async fn get_models_running(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models = {
        let context = "reading models of the runner";
        let guard = &state.runner;
        &guard.read_lock(context)?.models
    };
    let models = models
        .iter()
        .map(|model| ModelRuntimeDTO::from(model))
        .collect::<Vec<_>>();
    Ok(Json(json!(models)))
}

pub async fn run_model(
    State(state): State<Arc<AppState>>,
    Json(model): Json<ModelRuntimeDTO>,
) -> ResultAPI {
    let result = {
        let context = "launching model";
        let mut guard = state.runner.write_lock(context)?;
        &guard.load_model_name(&model.name)
    };
    let json = match result {
        Ok(_) => {
            json!([
                ("status".to_string(), "ok".to_string()),
                (
                    "message".to_string(),
                    "Model started successfully".to_string()
                ),
            ])
        }
        Err(err) => {
            json!([
                ("status".to_string(), "failed".to_string()),
                ("message".to_string(), err.to_string()),
            ])
        }
    };
    Ok(Json(json))
}
