use crate::cache::k_v_cache::k_v_cache::{ArcCacheItem, ArcCacheList, KVCache};
use crate::config::config_models::qwen3::Qwen3Config;
use crate::error::{Error, Result};
use crate::factory::mask::create_attention_mask;
use crate::mask::mask::AttentionMask;
use crate::model::model::Model;
use crate::model::models::qwen3::transformer_block::TransformerBlockQwen3;
use crate::model::weight::{Tensor, Weight};
use crate::module::Module;
use crate::quantized::Quantize;
use crate::utils::maybe_quantized::{MaybeQuantizedEmbedding, MaybeQuantizedLinear};
use crate::utils::rms_norm::NormExt;
use mlx_rs::Array;
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::RmsNorm;
use mlx_rs::nn::{Embedding, Linear, LinearBuilder, RmsNormBuilder};
use mlx_rs::quantization::{MaybeQuantized, Quantizable};
use rayon::prelude::*;
use sn_core::utils::rw_lock::RwLockExt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct ModelQwen3 {
    pub qwen3_config: Rc<Qwen3Config>,
    pub layers: Vec<TransformerBlockQwen3>,
    pub norm: RmsNorm,
    pub lm_head: MaybeQuantized<Linear>,
    pub embed_tokens: MaybeQuantized<Embedding>,
    pub bytes: u64,
}

impl Quantize for ModelQwen3 {
    fn quantize(&mut self, _: i32, _: i32) -> Result<()> {
        let mut bits = 4;
        let mut group_size = 64;

        if let Some(quantization) = &self.qwen3_config.quantization {
            group_size = quantization.group_size;
            bits = quantization.bits;
        }

        self.lm_head = self.lm_head.clone().try_into_quantized(group_size, bits)?;
        self.embed_tokens = self
            .embed_tokens
            .clone()
            .try_into_quantized(group_size, bits)?;
        for layer in &mut self.layers {
            layer.quantize(group_size, bits)?;
        }
        Ok(())
    }
}

impl Module for ModelQwen3 {
    fn forward(
        &mut self,
        _: &Array,
        _: Option<&AttentionMask>,
        _: Option<ArcCacheItem>,
    ) -> Result<Array> {
        unimplemented!()
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        self.bytes += tensor.size;
        match name {
            //"lm_head.weight" => return Ok(self.lm_head.update_weight(&tensor.data)),
            //"lm_head.scales" => return Ok(self.lm_head.update_scales(&tensor.data)),
            //"lm_head.biases" => return Ok(self.lm_head.update_biases(&tensor.data)),
            "model.embed_tokens.weight" => {
                return Ok(self.embed_tokens.update_weight(&tensor.data));
            }
            "model.embed_tokens.scales" => {
                return Ok(self.embed_tokens.update_scales(&tensor.data));
            }
            "model.embed_tokens.biases" => {
                return Ok(self.embed_tokens.update_biases(&tensor.data));
            }
            "model.norm.weight" => return Ok(self.norm.update_weight(&tensor.data)),
            _ => {
                if name.starts_with("model.layers.") {
                    let parts: Vec<&str> = name.split(".").collect();
                    if parts.len() >= 5 {
                        let idx = parts[2].parse::<usize>()?;
                        if idx < self.layers.len() {
                            return Ok(self.layers[idx].set_weight(name, tensor)?);
                        }
                    }
                }
            }
        }
        Err(Error::UnsupportedWeight(name.to_string()))
    }
}

impl Model for ModelQwen3 {
    fn sanitize(&mut self, weight: &mut Weight) {
        if self.qwen3_config.tie_word_embeddings {
            weight.tensors.remove("lm_head.weight");
        }
    }

    fn supports_quantization(&self) -> bool {
        self.qwen3_config.quantization.is_some()
    }

    fn load_weights(&mut self, weight: &Weight) -> Result<()> {
        for (name, tensor) in &weight.tensors {
            self.set_weight(name.as_str(), tensor)?
        }
        Ok(())
    }

    fn get_num_layer(&self) -> usize {
        self.layers.len()
    }

    fn forward_model(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        caches: Option<ArcCacheList>,
    ) -> Result<Array> {
        let mut h = self.embed_tokens.forward(&x)?;
        let default_cache: Vec<Arc<RwLock<KVCache>>> = (0..self.layers.len())
            .into_iter()
            .map(|_| Arc::new(RwLock::new(KVCache::default())))
            .collect();

        let default_cache = Arc::new(RwLock::new(default_cache));

        let caches = caches.unwrap_or(default_cache);

        let default_mask = create_attention_mask(&h, None, false)?;
        let mask = match mask {
            Some(_) => mask,
            _ => Some(&default_mask),
        };

        for (i, layer) in self.layers.iter_mut().enumerate() {
            let context = format!("reding cache for layer {}", i);
            if let Some(cache) = caches.read_lock(context.as_str())?.get(i) {
                h = layer.forward(&h, mask, Some(cache.clone()))?;
            } else {
                h = layer.forward(&h, mask, None)?;
            }
        }

        let out = self.norm.forward(&h)?;
        if self.qwen3_config.tie_word_embeddings {
            Ok(self.embed_tokens.as_linear(&out)?)
        } else {
            Ok(self.lm_head.forward(&out)?)
        }
    }

    fn get_model_bytes(&self) -> u64 {
        self.bytes
    }
}

impl ModelQwen3 {
    pub fn new(qwen3_config: Rc<Qwen3Config>) -> Result<ModelQwen3> {
        let layers = (0..qwen3_config.num_hidden_layers)
            .map(|_| TransformerBlockQwen3::new(qwen3_config.clone()))
            .collect::<Result<Vec<_>>>()?;

        let norm = RmsNormBuilder {
            dimensions: qwen3_config.hidden_size,
            eps: qwen3_config.rms_norm_eps,
        }
        .build()?;

        let lm_head = MaybeQuantized::new(
            LinearBuilder {
                input_dims: qwen3_config.hidden_size,
                output_dims: qwen3_config.vocab_size,
                bias: false,
            }
            .build()?,
        );

        let embed_tokens = MaybeQuantized::new(Embedding::new(
            qwen3_config.vocab_size,
            qwen3_config.hidden_size,
        )?);

        Ok(ModelQwen3 {
            qwen3_config,
            layers,
            norm,
            lm_head,
            embed_tokens,
            bytes: 0,
        })
    }
}
