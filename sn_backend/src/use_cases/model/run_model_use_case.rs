use crate::{
    domain::model::value_object::RunModelOutput,
    error::{ErrorBackend, Result},
    utils::stream_channel::StreamChannel,
};
use std::sync::{Arc, RwLock};

use sn_core::{
    server::payload::backend::{
        run_model_metadata_response_sse::RunModelMetadataResponseSSE,
        run_model_request::RunModelRequest,
    },
    types::stream_data::StreamData,
    utils::rw_lock::RwLockExt,
};
use sn_inference::runner::Runner;
use tracing::error;

pub struct RunModelUseCase {
    runner: Arc<RwLock<Runner>>,
}

impl RunModelUseCase {
    pub fn new(runner: Arc<RwLock<Runner>>) -> Self {
        RunModelUseCase { runner }
    }

    pub async fn run_model(
        &self,
        stream: Option<&StreamChannel>,
        req: RunModelRequest,
    ) -> Result<RunModelOutput> {
        let model_name = req.get_model_name()?;
        let rx = match stream {
            Some(stream) => Some(stream.rx.clone()),
            None => None,
        };
        let tx = match stream {
            Some(stream) => Some(stream.tx.clone()),
            None => None,
        };
        let runner = self.runner.clone();
        let task = tokio::spawn(async move {
            let tx_err = tx.clone();
            let guard = runner.read_lock("launching model")?;
            let run_model_result = guard.load_model_name(model_name.as_ref(), tx.clone());

            if let (Err(e), Some(tx_err)) = (&run_model_result, tx_err) {
                error!("{}", e);
                let error = format!("Failed to generate text: {}", e);
                let _ = tx_err.send(StreamData::for_stream_error(error).into());
            }
            let model_id = run_model_result?;
            if let Some(tx) = tx {
                let _ = tx.send(StreamData::for_metadata_run_model_sse_response(
                    RunModelMetadataResponseSSE {
                        model_id: model_id.clone().into(),
                    },
                ));
            }
            Ok::<_, ErrorBackend>(model_id)
        });

        let output = match stream {
            None => {
                let task_result = task.await??;
                RunModelOutput::Json(task_result)
            }
            Some(_) => {
                let fut = Box::pin(async move {
                    let task_result = task.await??;
                    Ok::<_, ErrorBackend>(task_result)
                });
                RunModelOutput::Streaming {
                    receiver: rx,
                    completion: Some(fut),
                }
            }
        };
        Ok(output)
    }
}
