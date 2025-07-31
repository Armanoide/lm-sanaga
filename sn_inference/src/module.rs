use crate::cache::k_v_cache::ArcCacheItem;
use crate::error::Result;
use crate::mask::mask::AttentionMask;
use crate::model::weight::Tensor;
use crate::quantized::Quantize;
use mlx_rs::{Array, Stream};
use std::any::Any;
use std::sync::Arc;

pub trait Module: Any + Quantize {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
        stream: Option<Arc<Stream>>
    ) -> Result<Array>;

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()>;
}
