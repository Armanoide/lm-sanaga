use crate::cache::k_v_cache::k_v_cache::KVCache;
use crate::error::Result;
use crate::mask::mask::AttentionMask;
use mlx_rs::Array;
use std::sync::Arc;

pub fn scaled_dot_product_attention(
    queries: &Array,
    keys: &Array,
    values: &Array,
    _: Option<Arc<&KVCache>>,
    scale: f32,
    mask: Option<&AttentionMask>,
) -> Result<Array> {
    if let Some(att) = mask {
        Ok(mlx_rs::fast::scaled_dot_product_attention(
            queries,
            keys,
            values,
            scale,
            att.to_scaled_mask_opt()?,
        )?)
    } else {
        Ok(mlx_rs::fast::scaled_dot_product_attention(
            queries, keys, values, scale, None,
        )?)
    }
}
