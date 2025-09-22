use crate::{
    domain::message::aggregate::MessageAggregate,
    error::{ErrorBackend, Result},
    utils::stream_channel::StreamChannel,
};
use sn_core::{types::stream_data::StreamData, utils::rw_lock::RwLockExt};
use sn_inference::runner::Runner;
use std::sync::{Arc, RwLock};
use tracing::error;

use crate::domain::message::value_object::GenerateTextOutput;

pub struct GenerateTextUseCase {
    runner: Arc<RwLock<Runner>>,
}

impl GenerateTextUseCase {
    pub fn new(runner: Arc<RwLock<Runner>>) -> Self {
        GenerateTextUseCase { runner }
    }

    pub async fn generate(
        &self,
        stream: Option<&StreamChannel>,
        mut agg: MessageAggregate,
        model_id: Arc<str>,
        session_id: Option<i32>,
    ) -> Result<GenerateTextOutput> {
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
            let guard = runner.read_lock("reading runner for generate_text")?;
            let generate_text_result =
                guard.generate_text(&model_id, &agg.to_conversation_core()?, session_id, tx);
            if let (Err(e), Some(tx_err)) = (&generate_text_result, tx_err) {
                error!("{}", e);
                let error = format!("Failed to generate text: {}", e);
                let _ = tx_err.send(StreamData::for_stream_error(error).into());
            }
            let _ = agg.add_assistant_message(generate_text_result?);
            Ok::<_, ErrorBackend>(agg)
        });

        let output = match stream {
            None => {
                let task_result = task.await??;
                GenerateTextOutput::Json(task_result)
            }
            Some(_) => {
                let fut = Box::pin(async move {
                    let task_result = task.await??;
                    Ok::<_, ErrorBackend>(task_result)
                });
                GenerateTextOutput::Streaming {
                    receiver: rx,
                    completion: Some(fut),
                }
            }
        };
        Ok(output)
    }
    // async fn execute(&self) -> ResultAPIStream {
    //     let response = SseResponseBuilder::new(self.rx.clone()).build();
    //     let tx = self.tx.clone();
    //         let generate_text_result = if let Err(err) = generate_text_result {
    //             handle_error_generate_text(&err.to_string(), tx.clone());
    //             return;
    //         } else {
    //             //todo: handle the case where generate_text_result is None
    //             generate_text_result.unwrap()
    //         };
    //
    //         store_generate_text_result(state, payload, generate_text_result, Some(self.tx)).await;
    //     });
    //
    //     Ok(response?)
    // }

    // async fn execute(&self) -> ResultAPIStream {
    //     let response = SseResponseBuilder::new(self.rx.clone()).build();
    //     let runner = self.runner.clone();
    //     let model_id = self.model_id.clone();
    //     let conversation = self.conversation.clone();
    //     let session_id = self.session_id;
    //     let tx = self.tx.clone();
    //     let task = tokio::spawn(async move {
    //         let generate_text_result = (|| {
    //             let guard = runner.read_lock("reading runner for generate_text")?;
    //             let generate_text_result =
    //                 guard.generate_text(&model_id, &conversation, session_id, Some(tx))?;
    //             Ok::<_, ErrorBackend>(generate_text_result)
    //         })();
    //
    //         let generate_text_result = if let Err(err) = generate_text_result {
    //             handle_error_generate_text(&err.to_string(), tx.clone());
    //             return;
    //         } else {
    //             //todo: handle the case where generate_text_result is None
    //             generate_text_result.unwrap()
    //         };
    //
    //         store_generate_text_result(state, payload, generate_text_result, Some(self.tx)).await;
    //     });
    //
    //     Ok(response?)
    // }
}
