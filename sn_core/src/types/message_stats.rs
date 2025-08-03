use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
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

/*

  pub fn from_token_stream_manager(token_stream_manager: &TokenStreamManager) -> Result<TokenStats> {
      if let Some(token_generator) = &token_stream_manager.token_generator {
          let total_generated_tokens = {
              let context = "reading total_generated_tokens from token_generator";
              token_generator.read_lock(context)?.total_generated_tokens
          };
          let generation_duration = {
              let context = "reading generation_duration from token_generator";
              token_generator.read_lock(context)?.generation_duration
          };
          let prefill_duration = {
              let context = "reading prefill_duration from token_generator";
              token_generator.read_lock(context)?.prefill_duration
          };

          let generation_tps = total_generated_tokens as f64 / generation_duration;

          let prompt_tps = match prefill_duration {
              0.0 => 0.0,
              duration => total_generated_tokens as f64 / duration,
          };

          Ok(TokenStats {
              generation_duration,
              prompt_tps,
              generation_tps,
          })
      } else {
          Ok(TokenStats::default())
      }
   }

*/
