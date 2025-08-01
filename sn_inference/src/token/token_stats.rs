use sn_core::utils::rw_lock::RwLockExt;
use crate::token::token_stream_manager::TokenStreamManager;
use crate::error::Result;

#[derive(Debug, Clone, Default)]
pub struct TokenStats {
    generation_duration: f64,
    prompt_tps: f64,
    generation_tps: f64,
}

pub fn get_state(token_stream_manager: &TokenStreamManager) -> Result<(TokenStats)> {
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