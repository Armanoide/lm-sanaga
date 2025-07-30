use crate::cache::k_v_cache::ArcCacheItem;
use crate::error::Result;
use crate::mask::mask::AttentionMask;
use crate::model::weight::Tensor;
use crate::quantized::Quantize;
use mlx_rs::Array;
use std::any::Any;
pub trait Module: Any + Quantize {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array>;

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()>;
}
