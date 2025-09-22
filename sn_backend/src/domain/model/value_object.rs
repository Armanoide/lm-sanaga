use std::pin::Pin;

use crate::error::ErrorBackend;
use crossbeam::channel::Receiver;
use sn_core::types::stream_data::StreamData;

pub enum RunModelOutput {
    Json(String),
    Streaming {
        receiver: Option<Receiver<StreamData>>,
        completion: Option<Pin<Box<dyn Future<Output = Result<String, ErrorBackend>> + Send>>>,
    },
}

unsafe impl Send for RunModelOutput {}
unsafe impl Sync for RunModelOutput {}
