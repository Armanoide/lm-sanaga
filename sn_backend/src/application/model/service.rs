use crate::error::Result;
use std::sync::{Arc, RwLock};

use sn_core::{
    server::payload::backend::{
        list_running_model_response::ListRunningModelResponse, run_model_request::RunModelRequest,
    },
    utils::rw_lock::RwLockExt,
};
use sn_inference::runner::Runner;
use tracing::info;

use crate::{
    domain::model::value_object::RunModelOutput,
    use_cases::model::run_model_use_case::RunModelUseCase, utils::stream_channel::StreamChannel,
};

#[derive(Clone, Debug)]
pub struct ModelService {
    runner: Arc<RwLock<Runner>>,
}

impl ModelService {
    pub fn new(runner: Arc<RwLock<Runner>>) -> Self {
        Self { runner }
    }

    pub async fn stop_model(&self, req: RunModelRequest) -> Result<String> {
        let id = req.get_id()?;
        {
            let context = "stopping model";
            info!("Stopping model with ID: {}", id);
            self.runner.write_lock(context)?.unload_model(&id);
        }
        Ok(id)
    }

    pub async fn list_running_models(&self) -> Result<Vec<ListRunningModelResponse>> {
        let models = {
            let context = "reading models of the runner";
            let guard = &self.runner;
            &guard.read_lock(context)?.models.read_lock(context)?.clone()
        };
        let models = models
            .iter()
            .map(|model| ListRunningModelResponse {
                id: model.id.clone(),
                name: model.name.clone(),
            })
            .collect::<Vec<_>>();
        Ok(models)
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let models_installed: Vec<String> = {
            let context = "reading models installed of the runner";
            let guard = &self.runner;
            guard.read_lock(context)?.scan_model_installed()?
        };
        Ok(models_installed)
    }

    pub async fn run_model(&self, req: RunModelRequest) -> Result<RunModelOutput> {
        let use_case = RunModelUseCase::new(self.runner.clone());
        let stream = StreamChannel::new(&req.get_stream());
        let result = use_case.run_model(stream.as_ref(), req).await?;

        Ok(result)
    }
}
