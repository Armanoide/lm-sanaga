use std::sync::Arc;
use crate::cache::k_v_cache::KVCache;
use crate::mask::mask::AttentionMask;
use mlx_rs::{Array, Stream};
use mlx_rs::error::Exception;
pub fn scaled_dot_product_attention(
    queries: &Array,
    keys: &Array,
    values: &Array,
    _: Option<&KVCache>,
    scale: f32,
    mask: Option<&AttentionMask>,
    stream: Option<Arc<Stream>>
) -> Result<Array, Exception> {

    if let Some(stream) = stream {
        if let Some(m) = mask {
            Ok(mlx_rs::fast::scaled_dot_product_attention_device(
                queries,
                keys,
                values,
                scale,
                m.to_scaled_mask_opt(),
                stream.clone()
            )?)
        } else {
            Ok(mlx_rs::fast::scaled_dot_product_attention_device(
                queries, keys, values, scale, None, stream
            )?)
        }
    } else {
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
}
