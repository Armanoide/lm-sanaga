use std::pin::Pin;

use crate::{domain::message::aggregate::MessageAggregate, error::ErrorBackend};
use crossbeam::channel::Receiver;
use sn_core::types::stream_data::StreamData;

pub enum GenerateTextOutput {
    Json(MessageAggregate),
    Streaming {
        receiver: Option<Receiver<StreamData>>,
        completion:
            Option<Pin<Box<dyn Future<Output = Result<MessageAggregate, ErrorBackend>> + Send>>>,
    },
}

unsafe impl Send for GenerateTextOutput {}
unsafe impl Sync for GenerateTextOutput {}
