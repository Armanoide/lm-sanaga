use std::sync::{Arc, RwLock};
use crossbeam::channel::Sender;
use mlx_rs::{Array};
use mlx_rs::ops::{concatenate};
use mlx_rs::ops::indexing::{argmax_axis, IndexOp};
use mlx_rs::transforms::compile::clear_cache;
use rayon::prelude::*;
use crate::factory::k_v_cache::create_prompt_cache;
use crate::model::model::Model;
use crate::model::model_kind::ModelKind;
use crate::error::{Result};
use crate::cache::k_v_cache::{SNCacheList};
use crate::generator::generated_token_info::GeneratedTokenInfo;

pub type SamplerFn = Arc<dyn Fn(&Array) -> Result<Array> + Send + Sync> ;

type LogitsProcessor = Arc<dyn Fn(&Array, &Array) -> Result<Array> + Send + Sync> ;
use crate::generator::stream_buffer::StreamingBuffer;
use crate::utils::rw_lock::RwLockExt;

pub struct TokenGeneratorOpts {
    temperature: Option<f32>,
}

pub struct TokenGenerator {
    model: Arc<RwLock<ModelKind>>,
    pub cache: SNCacheList,
    sampler: SamplerFn,
    logits_processors: Vec<LogitsProcessor>,
    prefill_step_size: i32,
    max_tokens: usize,
    tokens: Option<Array>,
    prompt: Array,
    eot_ids: Vec<u32>,
    stop: bool,
    prompt_len: usize,
    b_tokens: StreamingBuffer,
    options: Option<TokenGeneratorOpts>,
    token_sender: Option<Sender<GeneratedTokenInfo>>
    //pub quantize_cache_fn: Box<dyn Fn(&mut Cache) + 'a>,
    //pub prompt_progress_callback: Box<dyn Fn(usize, usize) + 'a>,
}

impl TokenGenerator {
    pub fn new(model: Arc<RwLock<ModelKind>>, prompt: Vec<u32>, eot_ids: Vec<u32>, token_sender: Option<Sender<GeneratedTokenInfo>>, ) -> Result<TokenGenerator> {
        let default_sampler: SamplerFn = Arc::new(|x: &Array| {
            Ok(argmax_axis(&x, -1, false)?)
        });
        let max_tokens = 10000000;
        let prompt_len = prompt.len();
        let prompt = Array::from_slice(prompt.as_slice(), &[prompt_len as i32]);
        let context = "reading model to create a prompt cache";
        let cache = create_prompt_cache(&*model.read_lock(context)?);

        Ok(TokenGenerator {
            token_sender,
            cache,
            model: model.clone(),
            max_tokens,
            prefill_step_size: 250,
            logits_processors: Vec::new(),
            tokens: None,
            prompt_len,
            sampler: default_sampler,
            prompt,
            eot_ids,
            stop: false,
            b_tokens: StreamingBuffer::new(),
            options: None,
        })
    }

    fn model_call(&mut self, input_prompt: &Array, input_embeddings: Option<&Array>) -> Result<Array> {
            /*if let Some(emb) = input_embeddings {
                model.forward_with_embeddings(prompt, emb, &prompt_cache)
            } else {
                model.forward(prompt, &prompt_cache)
            }*/
        let context = "read model for forwarding";
        let result = self.model
            .write_lock(context)?
            .forward_model(&input_prompt, None, Option::from(self.cache.clone()))?;
        Ok(result)
    }

    fn step(&mut self, input_tokens: &Array, input_embeddings: Option<&Array>) -> Result<(Array, Array)>{

        let input_tokens_batched = input_tokens.flatten(None, None)?.expand_dims(0)?;

        let input_embeds_batched = if let Some(emb) = input_embeddings {
            Some(emb.expand_dims(0)?)
        } else {
            None
        };

        let mut logits = self.model_call(&input_tokens_batched, None/*input_embeds_batched.as_ref()*/)?;
        logits = logits.index((.., -1, ..));

        if !self.logits_processors.is_empty() {
            self.tokens = match &self.tokens {
                Some(t) => Some(concatenate(&[t, input_tokens])?),
                None => Some(input_tokens.clone()),
            };

            for processor in &self.logits_processors {
                logits = processor(self.tokens.as_ref().unwrap(), &logits)?;
            }
        }

        //quantize_cache_fn(&mut prompt_cache);

        let temperature = 1;

        let logits = &logits / temperature; // element-wise division

        let logprobs = &logits - &logits.logsumexp(true)?;
        let sampled = self.sampler.as_ref()(&logprobs)?;
        let result = (sampled, logprobs.squeeze_axes(&[0])?);
        Ok(result)
    }

    pub fn generate(&mut self, input_embeddings: Option<&Array>, ) -> Result<()> {
        let total_prompt_tokens = self.prompt.shape()[0];
        let mut prompt_processed_tokens = 0;
        let prefill_step_size = 2048;
        let mut prompt_input = self.prompt.clone();
        while total_prompt_tokens - prompt_processed_tokens > prefill_step_size {
            let prompt_chunk = prompt_input.index(0..self.prefill_step_size).expand_dims(0)?;
            let embed_slice = if let Some(emb) = input_embeddings {
                Some(emb.index(0..prefill_step_size).expand_dims(0)?)
            } else {
                None
            };

            self.model_call(&prompt_chunk, embed_slice.as_ref())?;
            //Todo: quantize_cache_fn(&mut prompt_cache);

            // Assume cache state is some vector of arrays
            //Todo: let _ = self.prompt_cache.state().iter().map(|s| s.eval()).collect::<Result<Vec<_>>>()?;

            //prompt_progress_callback(prompt_processed_tokens, total_prompt_tokens);
            prompt_processed_tokens += prefill_step_size;
            prompt_input = prompt_input.index(prefill_step_size..prompt_input.size() as i32);
            if let Some(emb) = input_embeddings {
                emb.index(prefill_step_size..emb.size() as i32);
            }
            clear_cache();
        }

        let (mut y, mut logprobs) = self.step(&prompt_input, input_embeddings)?;
        y.eval()?;
        logprobs.eval()?;

        let mut n = 0;
        loop {
            if n != self.max_tokens {
                let mut gti = GeneratedTokenInfo::default();
                gti.set_start_time(n, self.prompt_len);
                let (next_y, next_logprobs) = self.step(&y, None)?;
                next_y.eval()?;
                next_logprobs.eval()?;

                if n == 0 {
                    y.eval()?;
                    //prompt_progress_callback(total_prompt_tokens, total_prompt_tokens);
                }

                if n == self.max_tokens || self.stop {
                    println!("Reached max tokens or stop condition at n={}", n);
                    break;
                }

                let z= y.as_slice();
                gti.set_token(z);
                if let Some(sender) = &self.token_sender {
                    if let Err(e) = sender.send(gti) {
                        log::error!("Failed to send token through crossbeam: {}", e);
                    }
                }
                if z.par_iter().any(|tok| self.eot_ids.contains(tok)) {
                    println!("EOT token found at n={}", n);
                    break;
                }
                //let s = self.tokenizer.decode_response(&z);
                if n % 256 == 0 {
                    clear_cache();
                }
                y = next_y;
                logprobs = next_logprobs;
                n += 1;
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Iterator for TokenGenerator {
    type Item = Vec<Arc<RwLock<GeneratedTokenInfo>>>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.b_tokens.flush())
    }
}

unsafe impl Send for TokenGenerator {}
unsafe impl Sync for TokenGenerator {}