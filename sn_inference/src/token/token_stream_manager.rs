use crate::error::{Error, Result};
use crate::model::model_kind::ModelKind;
use crate::token::token_generated_info::TokenGeneratedInfo;
use crate::token::token_generator::TokenGenerator;
use crate::tokenizer::tokenizer::Tokenizer;
use crossbeam::channel::{Receiver, Sender, bounded};
use rayon::prelude::*;
use sn_core::utils::rw_lock::RwLockExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::{debug, error};

pub type PromptStreamCallback = Sender<String>;
pub struct TokenStreamManager {
    tokenizer: Rc<Tokenizer>,
    pub token_generator: Option<Arc<RwLock<TokenGenerator>>>,
    model: Arc<RwLock<ModelKind>>,
    stop: bool,
    prompt_len: usize,
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
            prompt_len: 0,
            responses: Vec::new(),
            token_receiver: None,
        }
    }

    fn prelude_generate_text(&mut self, prompt: Vec<u32>) -> Result<()> {
        let eot_ids = &self.tokenizer.eot_ids();
        let model = self.model.clone();

        let (tx, rx): (Sender<TokenGeneratedInfo>, Receiver<TokenGeneratedInfo>) = bounded(100);
        self.token_receiver = Some(rx);

        // Create TokenGenerator on main thread to avoid error
        let tg = TokenGenerator::new(model, prompt, eot_ids.clone(), Some(tx))?;

        // Set token_generator so it can be used later
        let tg_arc = Arc::new(RwLock::new(tg));
        self.token_generator = Some(tg_arc.clone());

        // Spawn thread that just calls generate
        let handle = thread::spawn(move || {
            if let Ok(mut tg) = tg_arc.write_lock("threaded_generate") {
                debug!("Thread started to generate tokens.");
                let _ = tg.generate(None);
            } else {
                error!("Failed to acquire write lock in thread.");
            }
        });
        println!("Token generation thread spawned.");
        Ok(())
    }

    pub fn generate_text(
        &mut self,
        prompt: Vec<u32>,
        callback: Option<PromptStreamCallback>,
    ) -> Result<()> {
        self.prelude_generate_text(prompt)?;
        let eot_ids = self.tokenizer.eot_ids();
        let mut stop = false;

        if let Some(_generator) = &mut self.token_generator {
            // Take ownership of the receiver once, before the loop
            let rx = self
                .token_receiver
                .take()
                .ok_or_else(|| Error::TokenGenerationStartFailure)?;

            println!("Waiting for tokens...");
            for mut gti in rx.iter() {
                // Timing inside the prompt prefill loop
                let _step_start = Instant::now();

                // Check if any token matches EOT
                stop = gti.token.par_iter().any(|tok| eot_ids.contains(tok));

                self.tokenizer
                    .decode_response_from_generated_token_info(&mut gti);

                if let Some(cb) = &callback {
                    // Call the callback with the decoded response
                    let _ = cb.send(gti.text.clone());
                }

                if let Err(e) = gti.end(None) {
                    error!("Could not set the end time for the generated token: {}", e);
                }

                self.responses.push(gti);

                if stop {
                    break;
                }
            }
        } else {
            return Err(Error::TokenGenerationStartFailure);
        }



        Ok(())
    }
}
