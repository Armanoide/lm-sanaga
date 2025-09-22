use std::sync::Arc;

use crossbeam::channel::{bounded, Receiver, Sender};
use sn_core::types::stream_data::StreamData;

pub struct StreamChannel {
    pub tx: Arc<Sender<StreamData>>,
    pub rx: Receiver<StreamData>,
}

impl StreamChannel {
    pub fn new(use_stream: &Option<bool>) -> Option<StreamChannel> {
        match use_stream.unwrap_or(false) {
            true => {
                let (tx, rx) = bounded::<StreamData>(100);
                return Some(StreamChannel {
                    tx: Arc::new(tx),
                    rx,
                });
            }
            false => None,
        }
    }
}
