use crate::error::{Error, Result};
use crate::model::model::{ForwardType, Model};
use crate::model::model_kind::ModelKind;
use crate::token::token_generated_info::TokenGeneratedInfo;
use crossbeam::channel::Sender;
use mlx_rs::Array;
use mlx_rs::ops::concatenate;
use mlx_rs::ops::indexing::{IndexOp, argmax_axis};
use mlx_rs::transforms::async_eval;
use mlx_rs::transforms::compile::clear_cache;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::{debug, error, warn};

pub type SamplerFn = Arc<dyn Fn(&Array) -> Result<Array> + Send + Sync>;

type LogitsProcessor = Arc<dyn Fn(&Array, &Array) -> Result<Array> + Send + Sync>;
use crate::cache::k_v_cache::k_v_cache::ArcCacheList;
use crate::utils::mlx::mlx_compute_lock::MLX_COMPUTE_LOCK;
use sn_core::utils::rw_lock::RwLockExt;

/*fn top_k_filter(logits: &Array, k: usize) -> Result<Array> {
    // Convert to a Vec for sorting
    let mut logits_vec = logits.as_slice::<f32>().to_vec();

    // Find the k-th largest logit
    let mut sorted = logits_vec.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let kth_value = *sorted.get(k - 1).unwrap_or(&f32::MIN);

    // Mask logits below the k-th largest
    for logit in logits_vec.iter_mut() {
        if *logit < kth_value {
            *logit = f32::NEG_INFINITY;
        }
    }

    // Convert back into Array
    let shape = logits.shape();
    Ok(Array::from_slice(
        logits_vec.as_ref(),
        &[shape[0], shape[1]],
    ))
}*/

#[derive(Clone, Debug, Default)]
pub struct TokenGeneratorOpts {
    temperature: Option<f32>,
    //max_tokens: Option<usize>,
    //top_k: Option<usize>,
    //top_p: Option<f32>,
    //repetition_penalty: Option<f32>,
    //presence_penalty: Option<f32>,
    //frequency_penalty: Option<f32>,
}

pub struct TokenGenerator {
    model: Arc<RwLock<ModelKind>>,
    pub cache: ArcCacheList,
    sampler: SamplerFn,
    logits_processors: Vec<LogitsProcessor>,
    max_tokens: usize,
    tokens: Option<Array>,
    prompt: Array,
    eot_ids: HashSet<u32>,
    stop: bool,
    options: TokenGeneratorOpts,
    token_sender: Option<Sender<TokenGeneratedInfo>>,
    pub total_generated_tokens: usize,
    pub prefill_duration: f64,
    pub generation_duration: f64,
}

impl TokenGenerator {
    pub fn new(
        model: Arc<RwLock<ModelKind>>,
        prompt: Vec<u32>,
        eot_ids: HashSet<u32>,
        cache: ArcCacheList,
        token_sender: Option<Sender<TokenGeneratedInfo>>,
    ) -> Result<TokenGenerator> {
        let default_sampler: SamplerFn = Arc::new(|x: &Array| Ok(argmax_axis(&x, -1, false)?));
        let max_tokens = 10000000;
        let prompt_len = prompt.len();
        let prompt = Array::from_slice(prompt.as_slice(), &[prompt_len as i32]);

        Ok(TokenGenerator {
            token_sender,
            cache,
            model: model.clone(),
            max_tokens,
            logits_processors: Vec::new(),
            tokens: None,
            sampler: default_sampler,
            prompt,
            eot_ids,
            stop: false,
            options: TokenGeneratorOpts::default(),
            total_generated_tokens: 0,
            generation_duration: 0.0,
            prefill_duration: 0.0,
        })
    }

    fn model_call(
        &mut self,
        input_prompt: &Array,
        _: Option<&Array>, // input_embeddings
    ) -> Result<Array> {
        let context = "TokenGenerator:model_call";
        let result = self.model.write_lock(context)?.forward_model(
            &input_prompt,
            None,
            Option::from(self.cache.clone()),
            &ForwardType::Logits,
        )?;
        Ok(result)
    }

    fn forward_step(
        &mut self,
        input_tokens: &Array,
        input_embeddings: Option<&Array>,
    ) -> Result<(Array, Array)> {
        let input_tokens_batched = input_tokens.flatten(None, None)?.expand_dims(0)?;

        let input_embeds_batched = if let Some(emb) = input_embeddings {
            Some(emb.expand_dims(0)?)
        } else {
            None
        };

        let mut logits = self.model_call(&input_tokens_batched, input_embeds_batched.as_ref())?;
        logits = logits.index((.., -1, ..));

        if !self.logits_processors.is_empty() {
            self.tokens = match &self.tokens {
                Some(t) => Some(concatenate(&[t, input_tokens])?),
                None => Some(input_tokens.clone()),
            };

            for processor in &self.logits_processors {
                if let Some(tokens) = &self.tokens {
                    logits = processor(tokens, &logits)?;
                }
            }
        }

        //Todo: quantize_cache_fn(&mut prompt_cache);

        let temperature = self.options.temperature.unwrap_or(1.0);

        let logits = &logits / temperature; // element-wise division

        // Convert logits to mutable array
        let logits_mut = logits.clone();

        // Apply repetition penalty
        /*if let Some(tokens) = &self.tokens {
            let penalty = 1.2; // tweak as needed
            for token_id in tokens.as_slice::<i32>() {
                let val = logits.index(*token_id) / penalty;
                //  logits_mut.index_mut(token_id, val);
            }
        }*/

        // Apply top-k filter
        //logits_mut = top_k_filter(&logits_mut, 50)?;

        let logprobs = &logits_mut - &logits_mut.logsumexp(true)?;
        let sampled = self.sampler.as_ref()(&logprobs)?;
        let result = (sampled, logprobs.squeeze_axes(&[0])?);
        Ok(result)
    }

    fn step_prefill(
        &mut self,
        mut prompt_input: Array,
        input_embeddings: Option<&Array>,
    ) -> Result<Array> {
        let total_prompt_tokens = self.prompt.shape()[0];
        let mut prompt_processed_tokens = 0;
        let prefill_step_size = 256 / 2; // 128 tokens per step

        debug!(
            "will use prefill with {} ",
            total_prompt_tokens - prompt_processed_tokens > prefill_step_size
        );
        while total_prompt_tokens - prompt_processed_tokens > prefill_step_size {
            let prompt_chunk = prompt_input.index(0..prefill_step_size).expand_dims(0)?;
            let embed_slice = if let Some(emb) = input_embeddings {
                Some(emb.index(0..prefill_step_size).expand_dims(0)?)
            } else {
                None
            };

            self.model_call(&prompt_chunk, embed_slice.as_ref())?;
            //Todo: quantize_cache_fn(&mut prompt_cache);

            // Assume cache state is some vector of arrays
            {
                let context = "reading cache list";
                self.cache
                    .read_lock(context)?
                    .par_iter()
                    .for_each(|cache_item_lock| {
                        cache_item_lock
                            .write_lock("reading cache state")
                            .ok()
                            .map(|cache_item| cache_item.eval_state());
                    })
            };

            prompt_processed_tokens += prefill_step_size;
            prompt_input = prompt_input.index(prefill_step_size..);

            if let Some(emb) = input_embeddings {
                emb.index(prefill_step_size..emb.size() as i32);
            }
        }
        //clear_cache();
        Ok(prompt_input)
    }

    pub fn generate(&mut self, input_embeddings: Option<&Array>) -> Result<()> {
        let prompt_input = self.prompt.clone();
        let pre_fill_start = Instant::now();
        let prompt_input = self.step_prefill(prompt_input, input_embeddings)?;
        let (mut y, logprobs) = self.forward_step(&prompt_input, input_embeddings)?;
        {
            let _guard = MLX_COMPUTE_LOCK
                .lock()
                .map_err(|e| Error::MLXComputeLock(e.to_string()))?;
            async_eval([&y, &logprobs])?;
        }
        self.prefill_duration = pre_fill_start.elapsed().as_secs_f64();
        let mut generation_start = Instant::now();
        let mut n = 0;
        loop {
            if n != self.max_tokens {
                let mut gti = TokenGeneratedInfo::default();
                if n == 0 {
                    generation_start = Instant::now();
                }
                let (next_y, next_logprobs) = self.forward_step(&y, None)?;
                {
                    let _guard = MLX_COMPUTE_LOCK
                        .lock()
                        .map_err(|e| Error::MLXComputeLock(e.to_string()))?;
                    async_eval([&next_y, &next_logprobs])?;
                }
                if n == 0 {
                    let _guard = MLX_COMPUTE_LOCK
                        .lock()
                        .map_err(|e| Error::MLXComputeLock(e.to_string()))?;
                    y.eval()?;
                }
                if n == self.max_tokens || self.stop {
                    warn!("Reached max tokens or stop condition at n={}", n);
                    break;
                }

                let z = y.as_slice();
                gti.set_token(z, n);
                let z = gti.get_token().clone();
                if let Some(sender) = &self.token_sender {
                    if let Err(e) = sender.send(gti) {
                        error!("Failed to send token through crossbeam: {}", e);
                    }
                }
                if self.eot_ids.contains(&z) {
                    break;
                }
                if n % 256 == 0 {
                    clear_cache();
                }
                y = next_y;
                n += 1;

                self.total_generated_tokens += 1;
                self.generation_duration = generation_start.elapsed().as_secs_f64();
            } else {
                break;
            }
        }
        Ok(())
    }
}

unsafe impl Send for TokenGenerator {}
unsafe impl Sync for TokenGenerator {}
