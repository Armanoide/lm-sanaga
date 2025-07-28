use crate::cache::k_v_cache::{ArcCacheItem, ArcCacheList};
use sn_core::error::{Error, Result};
use crate::mask::mask::AttentionMask;
use crate::model::model::Model;
use crate::model::models::llama::llama::ModelLLama;
use crate::model::weight::{Tensor, Weight};
use crate::module::Module;
use crate::quantized::Quantize;
use mlx_rs::Array;

pub enum ModelKind {
    LLaMA(ModelLLama),
}

impl Module for ModelKind {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array> {
        match self {
            ModelKind::LLaMA(m) => m.forward(x, mask, cache),
        }
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        match self {
            ModelKind::LLaMA(m) => m.set_weight(name, tensor),
        }
    }
}

impl Quantize for ModelKind {
    fn quantize(&mut self, group_size: i32, bits: i32) -> std::result::Result<(), Error> {
        match self {
            ModelKind::LLaMA(m) => m.quantize(group_size, bits),
        }
    }
}

impl Model for ModelKind {
    fn sanitize(&mut self, weight: &mut Weight) {
        match self {
            ModelKind::LLaMA(m) => m.sanitize(weight),
        }
    }

    fn supports_quantization(&self) -> bool {
        match self {
            ModelKind::LLaMA(m) => m.supports_quantization(),
        }
    }

    fn load_weights(&mut self, weight: &Weight) -> Result<()> {
        match self {
            ModelKind::LLaMA(m) => m.load_weights(weight),
        }
    }

    fn get_num_layer(&self) -> usize {
        match self {
            ModelKind::LLaMA(m) => m.get_num_layer(),
        }
    }

    fn forward_model(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        caches: Option<ArcCacheList>,
    ) -> Result<Array> {
        match self {
            ModelKind::LLaMA(m) => m.forward_model(x, mask, caches),
        }
    }

    fn get_model_bytes(&self) -> u64 {
        match self {
            ModelKind::LLaMA(m) => m.get_model_bytes(),
        }
    }
}
