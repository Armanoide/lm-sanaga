use crate::mask::mask::AttentionMask;
use mlx_rs::Array;
use mlx_rs::error::Exception;
use crate::cache::k_v_cache::k_v_cache::KVCache;

pub fn scaled_dot_product_attention(
    queries: &Array,
    keys: &Array,
    values: &Array,
    _: Option<&KVCache>,
    scale: f32,
    mask: Option<&AttentionMask>,
) -> Result<Array, Exception> {
    if let Some(m) = mask {
        Ok(mlx_rs::fast::scaled_dot_product_attention(
            queries,
            keys,
            values,
            scale,
            m.to_scaled_mask_opt(),
        )?)
    } else {
        Ok(mlx_rs::fast::scaled_dot_product_attention(
            queries, keys, values, scale, None,
        )?)
    }
}
