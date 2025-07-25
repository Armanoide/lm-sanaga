use std::time::{SystemTime, UNIX_EPOCH};
use crate::utils::mlx::get_peak_memory::get_peak_memory;
use crate::error::Result;

#[derive(Debug, Default)]
pub struct GeneratedTokenInfo {
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

impl GeneratedTokenInfo {
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


pub trait TokenStatsExt {
    fn tokens_stats(&self, skip_first: bool) -> Option<(f64, f64, f64)>;
}

impl TokenStatsExt for Vec<GeneratedTokenInfo> {
    fn tokens_stats(&self, skip_first: bool) -> Option<(f64, f64, f64)> {
        let data = if skip_first && self.len() > 1 {
            &self[1..]
        } else {
            self.as_slice()
        };

        if data.is_empty() {
            return None;
        }

        let len = data.len() as f64;

        let (total_gen_tps, total_prompt_tps, total_peak_mem): (f64, f64, f64) = data.iter()
            .map(|info| (
                info.generation_tps,
                info.prompt_tps,
                info.peak_memory as f64))
            .fold((0.0, 0.0, 0.0), |acc, x| (
                acc.0 + x.0,
                acc.1 + x.1,
                acc.2 + x.2,
            ));

        Some((
            /*avg_generation_tps:*/ total_gen_tps / len,
            /*avg_prompt_tps:*/ total_prompt_tps / len,
            /* avg_peak_memory:*/ total_peak_mem / len,
        ))
    }
}
