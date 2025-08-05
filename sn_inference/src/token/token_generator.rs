use std::collections::HashSet;
use crate::cache::k_v_cache::ArcCacheList;
use crate::error::{Error, Result};
use crate::factory::k_v_cache::create_prompt_cache;
use crate::model::model::Model;
use crate::model::model_kind::ModelKind;
use crate::token::token_generated_info::TokenGeneratedInfo;
use crate::utils::mlx::metal_device_info::metal_device_info;
use crate::utils::mlx::metal_is_available::metal_is_available;
use crate::utils::mlx::set_wired_limit::set_wired_limit;
use crossbeam::channel::Sender;
use mlx_rs::Array;
use mlx_rs::ops::concatenate;
use mlx_rs::ops::indexing::{IndexOp, argmax_axis};
use mlx_rs::transforms::{async_eval, async_eval_params, eval, eval_params};
use mlx_rs::transforms::compile::clear_cache;
use rayon::prelude::*;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use once_cell::unsync::Lazy;
use tokio::sync::Mutex;

pub type SamplerFn = Arc<dyn Fn(&Array) -> Result<Array> + Send + Sync>;

type LogitsProcessor = Arc<dyn Fn(&Array, &Array) -> Result<Array> + Send + Sync>;
use sn_core::utils::rw_lock::RwLockExt;

/// A global mutex to **serialize access to MLX GPU compute**.
///
/// MLX/Metal does **not support concurrent compute operations**
/// (e.g., async_eval / eval), and doing so can cause
/// segmentation faults or GPU command buffer assertion failures.
///
/// This lock ensures that **only one thread/task** runs
/// compute operations like `async_eval` at a time.
static MLX_COMPUTE_LOCK: once_cell::sync::Lazy<std::sync::Mutex<bool>> = once_cell::sync::Lazy::new(|| {
    std::sync::Mutex::new(true)
});

pub struct TokenGeneratorOpts {
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    top_k: Option<usize>,
    top_p: Option<f32>,
    repetition_penalty: Option<f32>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
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
    prompt_len: usize,
    options: Option<TokenGeneratorOpts>,
    token_sender: Option<Sender<TokenGeneratedInfo>>,
    pub total_generated_tokens: usize,
    pub total_prompt_duration: f64,
    pub prefill_duration: f64,
    pub generation_duration: f64,
}


impl TokenGenerator {
    pub fn new(
        model: Arc<RwLock<ModelKind>>,
        prompt: Vec<u32>,
        eot_ids: HashSet<u32>,
        token_sender: Option<Sender<TokenGeneratedInfo>>,
    ) -> Result<TokenGenerator> {
        let default_sampler: SamplerFn = Arc::new(|x: &Array| Ok(argmax_axis(&x, -1, false)?));
        let max_tokens = 10000000;
        let prompt_len = prompt.len();
        let prompt = Array::from_slice(prompt.as_slice(), &[prompt_len as i32]);
        let cache = {
            let context = "reading model to create a prompt cache";
            create_prompt_cache(&*model.read_lock(context)?)
        };
        Ok(TokenGenerator {
            token_sender,
            cache,
            model: model.clone(),
            max_tokens,
            logits_processors: Vec::new(),
            tokens: None,
            prompt_len,
            sampler: default_sampler,
            prompt,
            eot_ids,
            stop: false,
            options: None,
            total_generated_tokens: 0,
            total_prompt_duration: 0.0,
            generation_duration: 0.0,
            prefill_duration: 0.0,
        })
    }

    fn model_call(
        &mut self,
        input_prompt: &Array,
        input_embeddings: Option<&Array>,
    ) -> Result<Array> {
        let context = "read model for forwarding";
        let result = self.model.write_lock(context)?.forward_model(
            &input_prompt,
            None,
            Option::from(self.cache.clone()),
        )?;
        Ok(result)
    }

    fn step(
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

        let mut logits = self.model_call(
            &input_tokens_batched,
            input_embeds_batched.as_ref()
        )?;
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

        let temperature = 1;

        let logits = &logits / temperature; // element-wise division

        let logprobs = &logits - &logits.logsumexp(true)?;
        let sampled = self.sampler.as_ref()(&logprobs)?;
        let result = (sampled, logprobs.squeeze_axes(&[0])?);
        Ok(result)
    }

    fn weird_limit(&self) -> Result<()> {
        /*println!("//metal_is_available= {}", metal_is_available()?);
        if metal_is_available()? {
            let model_bytes = {
                let context = "reading n_bytes of the model";
                self.model.read_lock(context)?.get_model_bytes()
            };
            println!("//model_bytes= {:?}", model_bytes);
            let device_info = metal_device_info()?;
            println!("{:?}", device_info);
            let max_recommended_size = device_info.max_recommended_working_set_size;
            if model_bytes > (0.9 * (max_recommended_size as f64)) as u64 {
                println!("//ici");
                let model_mb = model_bytes / 1 << 20;
                let max_rec_mb = max_recommended_size / 1 << 20;
                //let old_limit = set_wired_limit(max_recommended_size)?;
            }
        }*/
        //synchron
        Ok(())
    }

    pub fn generate(&mut self, input_embeddings: Option<&Array>) -> Result<()> {
        let total_prompt_tokens = self.prompt.shape()[0];
        let mut prompt_processed_tokens = 0;
        let prefill_step_size = 256 * 4;
        let mut prompt_input = self.prompt.clone();
        let pre_fill_start = Instant::now();

        /*match self.weird_limit() {
            Ok(_) => info!("Weird limit set successfully"),
            Err(e) => error!("Failed to set weird limit: {}", e),
        }*/

        while total_prompt_tokens - prompt_processed_tokens > prefill_step_size {
            let prompt_chunk = prompt_input
                .index(0..prefill_step_size)
                .expand_dims(0)?;
            let embed_slice = if let Some(emb) = input_embeddings {
                Some(emb.index(0..prefill_step_size).expand_dims(0)?)
            } else {
                None
            };

            self.model_call(&prompt_chunk, embed_slice.as_ref())?;
            //Todo: quantize_cache_fn(&mut prompt_cache);

            // Assume cache state is some vector of arrays
            let states: Vec<_> = {
                let context = "reading cache list";
                self.cache.read_lock(context)?.iter()
                    .filter_map(|cache_item_lock| {
                        cache_item_lock.read_lock("reading cache state").ok()
                            .map(|cache_item| cache_item.get_state())
                    })
                    .collect()
            };

            states.iter().for_each(|s| {
               if let Err(e) = eval([&s.0, &s.1]) {
                   error!("Failed to eval state cache during prompt process: {}", e);
               }
            });

            prompt_processed_tokens += prefill_step_size;
            prompt_input = prompt_input.index(prefill_step_size..);

            if let Some(emb) = input_embeddings {
                emb.index(prefill_step_size..emb.size() as i32);
            }
            clear_cache();
        }

        let (mut y, mut logprobs) = self.step(&prompt_input, input_embeddings)?;
        {
            let _guard = MLX_COMPUTE_LOCK.lock()
                .map_err(|e| Error::MLXComputeLock)?;
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
                let (next_y, next_logprobs) = self.step(&y, None)?;
                {
                    let _guard = MLX_COMPUTE_LOCK.lock()
                        .map_err(|e| Error::MLXComputeLock)?;
                    async_eval([&next_y, &next_logprobs])?;
                }
                if n == 0 {
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
                logprobs = next_logprobs;
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

/*

|-----|
|  A  | ------------>-------|
|-----|                     |
                            |
|-----|                     |
|  B  | ------------>  struct Mutex     ------> compute GPU
|-----|                     |
                            |
|-----|                     |
|  C  | ------------>-------|
|-----|


Mutex<

*/