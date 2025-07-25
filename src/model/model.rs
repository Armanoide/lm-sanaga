use crate::error::{Result};
use crate::module::{Module};
use mlx_rs::{Array};
use crate::mask::mask::AttentionMask;
use crate::model::weight::Weight;
use crate::cache::k_v_cache::{SNCacheList};

pub trait Model: Module {
    fn sanitize(&mut self, weight: &mut Weight);
    fn supports_quantization(&self) -> bool;
    fn load_weights(&mut self, weight: &Weight) -> Result<()>;
    fn get_num_layer(&self) -> usize;
    fn forward_model(&mut self, x: &Array, mask: Option<&AttentionMask>, caches: Option<SNCacheList>) -> Result<Array>;
    fn get_model_bytes(&self) -> u64;
}

