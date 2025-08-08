use crate::error::{Error, Result};
use crate::mask::mask::AttentionMask;
use crate::model::model::Model;
use crate::model::models::llama::llama::ModelLLama;
use crate::model::weight::{Tensor, Weight};
use crate::module::Module;
use crate::quantized::Quantize;
use mlx_rs::Array;
use crate::cache::k_v_cache::k_v_cache::{ArcCacheItem, ArcCacheList};
use crate::model::models::qwen3::qwen3::ModelQwen3;

macro_rules! delegate_to_variants {
    // For &self methods
    ($this:ident => $method:ident $(, $arg:expr)* ) => {
        match $this {
            ModelKind::LLaMA(m) => m.$method($($arg),*),
            ModelKind::Qwen3(m) => m.$method($($arg),*),
        }
    };

    // For &mut self methods
    (mut $this:ident => $method:ident $(, $arg:expr)* ) => {
        match $this {
            ModelKind::LLaMA(m) => m.$method($($arg),*),
            ModelKind::Qwen3(m) => m.$method($($arg),*),
        }
    };
}
#[derive(Debug)]
pub enum ModelKind {
    LLaMA(ModelLLama),
    Qwen3(ModelQwen3)
}

impl Module for ModelKind {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array> {
        delegate_to_variants!(mut self => forward, x, mask, cache)
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        delegate_to_variants!(mut self => set_weight, name, tensor)
    }
}

impl Quantize for ModelKind {
    fn quantize(&mut self, group_size: i32, bits: i32) -> std::result::Result<(), Error> {
        delegate_to_variants!(mut self => quantize, group_size, bits)
    }
}

impl Model for ModelKind {
    fn sanitize(&mut self, weight: &mut Weight) {
        delegate_to_variants!(mut self => sanitize, weight)
    }

    fn supports_quantization(&self) -> bool {
        delegate_to_variants!(self => supports_quantization)
    }

    fn load_weights(&mut self, weight: &Weight) -> Result<()> {
        delegate_to_variants!(mut self => load_weights, weight)
    }

    fn get_num_layer(&self) -> usize {
        delegate_to_variants!(self => get_num_layer)
    }

    fn forward_model(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        caches: Option<ArcCacheList>,
    ) -> Result<Array> {
        delegate_to_variants!(self => forward_model, x, mask, caches)
    }

    fn get_model_bytes(&self) -> u64 {
        delegate_to_variants!(self => get_model_bytes)
    }
}
