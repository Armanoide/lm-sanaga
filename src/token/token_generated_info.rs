use crate::error::Result;
use crate::utils::mlx::get_peak_memory::get_peak_memory;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct TokenGeneratedInfo {
    pub text: String,
    pub token: Vec<u32>,
    //pub logprobs: Array,
    pub from_draft: bool,
    pub prompt_tokens: usize,
    pub prompt_tps: f64,
    pub generation_tps: f64,
    pub generation_tokens: usize,
    pub peak_memory: usize,
    pub start_timestamp: u128,
    pub end_timestamp: u128,
    pub finish_reason: Option<String>,
}

impl TokenGeneratedInfo {
    pub fn set_token(&mut self, token: &[u32]) {
        self.token = Vec::from(token);
    }

    pub fn set_start_time(&mut self, generation_tokens: usize, prompt_tokens: usize) {
        let now = SystemTime::now();
        self.prompt_tokens = prompt_tokens;
        self.generation_tokens = generation_tokens;
        self.start_timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
    }

    pub fn set_text(&mut self, text: String) {
        //println!("[{}]{}{:?}",self.generation_tokens, text, self.token);
        print!("{}", text);
        self.text = text;
    }

    pub fn set_end_time(&mut self, with_reason: Option<String>) -> Result<()> {
        let now = SystemTime::now();
        self.finish_reason = with_reason;
        self.end_timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis();

        let duration_ms = (self.end_timestamp - self.start_timestamp) as f64;
        let duration_secs = duration_ms / 1000.0;

        // Compute prompt TPS and generation TPS
        let prompt_tokens_f64 = self.prompt_tokens as f64;
        let generation_tokens_f64 = self.generation_tokens as f64;

        self.prompt_tps = if self.prompt_tokens > 0 {
            prompt_tokens_f64 / duration_secs
        } else {
            0.0
        };

        self.generation_tps = if self.generation_tokens > 0 {
            generation_tokens_f64 / duration_secs
        } else {
            0.0
        };

        self.peak_memory = get_peak_memory()?;
        Ok(())
    }
}
