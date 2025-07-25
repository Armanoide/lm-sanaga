use std::any::Any;
use mlx_rs::Array;
use crate::mask::mask::AttentionMask;
use crate::model::weight::{Tensor};
use crate::quantized::{Quantize};
use crate::cache::k_v_cache::{SNCacheItem};
use crate::error::Result;
pub trait Module: Any + Quantize {
    fn forward(&mut self, x: &Array, mask: Option<&AttentionMask>, cache: Option<SNCacheItem>) -> Result<Array>;

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()>;
}
