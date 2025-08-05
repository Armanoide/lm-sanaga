use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageStats {
    pub generation_duration: f64,
    pub prompt_tps: f64,
    pub generation_tps: f64,
}

#[derive(Debug, Clone, Default)]
pub struct MessageStatsBuilder {
    total_generated_tokens: f64,
    generation_duration: f64,
    prefill_duration: f64,
}

impl MessageStatsBuilder {
    pub fn new() -> Self {
        MessageStatsBuilder::default()
    }

    pub fn with_total_generated_tokens(
        &mut self,
        total_generated_tokens: f64,
    ) -> &mut MessageStatsBuilder {
        self.total_generated_tokens = total_generated_tokens;
        self
    }

    pub fn with_generation_duration(&mut self, generation_duration: f64) -> &mut MessageStatsBuilder {
        self.generation_duration = generation_duration;
        self
    }

    pub fn with_prefill_duration(&mut self, prefill_duration: f64) -> &mut MessageStatsBuilder {
        self.prefill_duration = prefill_duration;
        self
    }

    pub fn build(&self) -> MessageStats {
        let generation_tps = self.total_generated_tokens / self.generation_duration;

        let prompt_tps = match self.prefill_duration {
            0.0 => 0.0,
            duration => self.total_generated_tokens / duration,
        };

        MessageStats {
            generation_tps,
            prompt_tps,
            generation_duration: self.generation_duration,
        }
    }
}
