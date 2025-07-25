use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use crate::generator::generated_token_info::GeneratedTokenInfo;

pub struct StreamingBuffer {
    buffer: VecDeque<Arc<RwLock<GeneratedTokenInfo>>>,
}

impl StreamingBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    pub fn push_slice(&mut self, generated_token_info: GeneratedTokenInfo) {
        self.buffer.extend(std::iter::once(Arc::new(RwLock::new(generated_token_info))));
    }

    pub fn flush(&mut self) -> Vec<Arc<RwLock<GeneratedTokenInfo>>> {
        self.buffer.drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}