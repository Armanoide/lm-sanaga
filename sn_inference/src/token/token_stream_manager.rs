use crate::cache::k_v_cache::k_v_cache::{ArcCacheList, CacheSize};
use crate::error::{Error, Result};
use crate::model::model_kind::ModelKind;
use crate::token::token_generated_info::TokenGeneratedInfo;
use crate::token::token_generator::TokenGenerator;
use crate::tokenizer::tokenizer::Tokenizer;
use crossbeam::channel::{Receiver, Sender, bounded};
use rayon::prelude::*;
use sn_core::types::message_stats::{MessageStats, MessageStatsBuilder};
use sn_core::types::stream_data::StreamData;
use sn_core::utils::rw_lock::RwLockExt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;
use tracing::{debug, error};

pub type PromptStreamCallback = Arc<Sender<StreamData>>;
pub struct TokenStreamManager {
    tokenizer: Rc<Tokenizer>,
    model: Arc<RwLock<ModelKind>>,
    pub token_generator: Option<Arc<RwLock<TokenGenerator>>>,
    stop: bool,
    responses: Vec<TokenGeneratedInfo>,
    token_receiver: Option<Receiver<TokenGeneratedInfo>>,
}

impl TokenStreamManager {
    pub fn new(model: Arc<RwLock<ModelKind>>, tokenizer: Rc<Tokenizer>) -> TokenStreamManager {
        TokenStreamManager {
            model,
            tokenizer,
            token_generator: None,
            stop: false,
            responses: Vec::new(),
            token_receiver: None,
        }
    }

    fn prelude_generate_text(&mut self, prompt: Vec<u32>, cache: ArcCacheList) -> Result<()> {
        let eot_ids = &self.tokenizer.eot_ids();
        let model = self.model.clone();

        let (tx, rx): (Sender<TokenGeneratedInfo>, Receiver<TokenGeneratedInfo>) = bounded(100);
        self.token_receiver = Some(rx);

        // Create TokenGenerator on main thread to avoid error
        let tg = TokenGenerator::new(model, prompt, eot_ids.clone(), cache, Some(tx))?;

        // Set token_generator so it can be used later
        let tg_arc = Arc::new(RwLock::new(tg));
        self.token_generator = Some(tg_arc.clone());

        // Spawn thread that just calls generate
        let _ = thread::spawn(move || {
            if let Ok(mut tg) = tg_arc.write_lock("threaded_generate") {
                debug!("Thread started to generate tokens.");
                let _ = tg.generate(None);
            } else {
                error!("Failed to acquire write lock in thread.");
            }
        });
        Ok(())
    }

    pub fn get_text(&self) -> String {
        self.responses
            .par_iter()
            .map(|gti| gti.text.clone())
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn generate_text(
        &mut self,
        prompt: Vec<u32>,
        cache: ArcCacheList,
        callback: Option<PromptStreamCallback>,
    ) -> Result<String> {
        self.prelude_generate_text(prompt, cache.clone())?;
        let eot_ids = self.tokenizer.eot_ids();
        let header_token_ids = self.tokenizer.header_token_ids();

        if let Some(_generator) = &mut self.token_generator {
            let rx = self
                .token_receiver
                .take()
                .ok_or_else(|| Error::TokenGenerationStartFailure)?;

            let mut has_header_start = false;
            let mut has_header_end = false;

            for mut gti in rx.iter() {
                // Timing inside the prompt prefill loop
                let _step_start = Instant::now();

                // Check if any token matches EOT
                self.stop = eot_ids.contains(gti.get_token());
                if !has_header_start && header_token_ids.contains(gti.get_token()) {
                    has_header_start = true;
                } else if !has_header_end
                    && has_header_start
                    && header_token_ids.contains(gti.get_token())
                {
                    has_header_end = true;
                }

                self.tokenizer.decode_response_from_generated_token_info(
                    &mut gti,
                    has_header_start,
                    has_header_end,
                );

                if let Some(cb) = &callback {
                    // Cal the callback with the decoded response
                    let _ = cb.send(StreamData::for_string(gti.text.clone()));
                }

                if let Err(e) = gti.end(None) {
                    error!("Could not set the end time for the generated token: {}", e);
                }

                self.responses.push(gti);

                if self.stop {
                    break;
                }
            }
        } else {
            return Err(Error::TokenGenerationStartFailure);
        }
        println!("cache size: {}", cache.cache_size());
        Ok(self.get_text())
    }

    pub fn get_average_stats(&self) -> Result<Option<MessageStats>> {
        if let Some(token_generator) = &self.token_generator {
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
            let stats = MessageStatsBuilder::new()
                .with_total_generated_tokens(total_generated_tokens as f64)
                .with_generation_duration(generation_duration)
                .with_prefill_duration(prefill_duration)
                .build();

            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }
}
