use crate::cache::k_v_cache::k_v_cache::KVCache;
use crate::mask::mask::AttentionMask;
use mlx_rs::Array;
use mlx_rs::error::Exception;
use mlx_rs::ops::arange;
use std::cmp::min;

pub fn create_causal_mask(
    n: i32,
    offset: i32,
    window_size: Option<i32>,
) -> Result<Array, Exception> {
    // Right indices: shape (1, offset + N)
    let rinds = arange::<_, f32>(None, offset + n, None)?;

    // Left indices: shape (N, 1)
    let linds = if offset > 0 {
        arange::<_, f32>(offset, offset + n, None)?.reshape(&[n, 1])?
    } else {
        rinds.clone()
    };

    // shape: (N,)
    let linds = linds.reshape(&[n, 1])?;
    // shape: (offset + N,)
    let rinds = rinds.reshape(&[1, offset + n])?;

    // Base causal mask: linds >= rinds
    let mut mask = linds.ge(&rinds)?;

    // Apply windowed attention if needed
    if let Some(w) = window_size {
        let rinds_plus_window = rinds.add(Array::from_int(1))?;
        let window_mask = linds.le(&rinds_plus_window)?;
        mask = mask.logical_and(&window_mask)?;
    }

    Ok(mask)
}

pub fn create_attention_mask(
    h: &Array,
    cache: Option<&KVCache>,
    mut return_array: bool,
) -> Result<AttentionMask, Exception> {
    let shape = h.shape();
    let t = shape[1]; // assume h has shape [B, T, D] ?

    if t > 1 {
        let mut offset = 0;
        let mut window_size = None;

        if let Some(c) = cache {
            offset = c.offset;
            if let Some(max_size) = c.max_size {
                window_size = Some(max_size);
                offset = min(max_size, offset);
                return_array = return_array || (offset + t > max_size);
            }
        }

        if return_array {
            return Ok(AttentionMask::MaskArray(create_causal_mask(
                t,
                offset,
                window_size,
            )?));
        } else {
            return Ok(AttentionMask::Causal);
        }
    }
    Ok(AttentionMask::Causal)
}
