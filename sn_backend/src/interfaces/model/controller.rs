use crate::domain::model::value_object::RunModelOutput;
use crate::error::{ErrorBackend, ResultAPI, ResultAPIStream};
use crate::server::app_state::AppState;
use crate::utils::sse_response_builder::SseResponseBuilder;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sn_core::server::payload::backend::run_model_request::RunModelRequest;
use sn_core::server::payload::backend::run_model_response::{
    RunModelAction, RunModelResponse, RunModelResponseJson,
};
use std::sync::Arc;

pub async fn list_models_handler(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models_installed = state.service_model.list_models().await?;
    Ok(Json(models_installed.into()))
}

pub async fn list_running_models_handler(State(state): State<Arc<AppState>>) -> ResultAPI {
    let models = state.service_model.list_running_models().await?;
    Ok(Json(json!(models)))
}

pub async fn run_model_handler(
    State(state): State<Arc<AppState>>,
    req: std::result::Result<Json<RunModelRequest>, JsonRejection>,
) -> ResultAPIStream {
    let req = req?.0;
    let result = state.service_model.run_model(req).await?;

    match result {
        RunModelOutput::Json(model_id) => {
            return Ok(Json(json!(RunModelResponse::Json(RunModelResponseJson {
                model_id,
                status: RunModelAction::Start
            })))
            .into_response());
        }
        RunModelOutput::Streaming { receiver, .. } => {
            if let Some(receiver) = receiver {
                return SseResponseBuilder::new(receiver).build();
            }
            return Err(ErrorBackend::FailedToRunModel(
                "No streaming receiver available".into(),
            ));
        }
    }
}

pub async fn stop_model_handler(
    State(state): State<Arc<AppState>>,
    req: Result<Json<RunModelRequest>, JsonRejection>,
) -> ResultAPI {
    let req = req?.0;
    let id = state.service_model.stop_model(req).await?;
    Ok(Json(json!(RunModelResponseJson {
        model_id: id,
        status: RunModelAction::Stop,
    })))
}
