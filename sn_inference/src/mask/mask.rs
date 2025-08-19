use crate::error::{Error, Result};
use mlx_rs::Array;
use mlx_rs::fast::ScaledDotProductAttentionMask;
use mlx_rs::ops::broadcast_to;
use std::borrow::Cow;
#[derive(Debug, Default, Clone)]
pub enum AttentionMask<'a> {
    #[default]
    Causal,
    Array(Cow<'a, Array>),
}

impl<'a> AttentionMask<'a> {
    pub fn from_array(arr: Cow<'a, Array>) -> Result<Self> {
        let normalized = normalize_and_broadcast_mask(&arr)?;
        Ok(AttentionMask::Array(Cow::Owned(normalized)))
    }
}
pub fn normalize_and_broadcast_mask(arr: &Array) -> Result<Array> {
    let batch_size = arr.shape()[0];
    let seq_len = arr.shape()[1];

    // Normalize shape to [B, 1, 1, T]
    let reshaped = match arr.shape().len() {
        2 => arr.reshape(&[batch_size, 1, 1, seq_len])?,
        3 => arr.reshape(&[batch_size, 1, seq_len, seq_len])?, // already has extra dim
        4 => arr.clone(),                                      // already in 4D
        _ => {
            return Err(Error::UnexpectedMaskShape(format!("{:?}", arr.shape())));
        }
    };
    // Broadcast to [B, 1, T, T]
    let broadcasted = broadcast_to(&reshaped, &[batch_size, 1, seq_len, seq_len])?;
    Ok(broadcasted)
}
impl<'a> AttentionMask<'a> {
    pub fn to_scaled_mask_opt(&self) -> Result<Option<ScaledDotProductAttentionMask<'_>>> {
        match self {
            AttentionMask::Causal => Ok(Some(ScaledDotProductAttentionMask::Causal)),
            AttentionMask::Array(arr) => {
                let arr = arr.as_ref();
                Ok(Some(ScaledDotProductAttentionMask::from(arr)))
            }
        }
    }
}

/*
pub fn bitwise_and_arrays(a: &Array, b: &Array) -> Result<Array> {
    let device = Stream::default();

    // create an uninitialized array with the correct shape and dtype
    let mut result = Array::zeros::<bool>(&a.shape())?;
    let mut res: mlx_array = unsafe { mlx_array_new() };
    // unsafe call to fill result data
    unsafe {
        mlx_bitwise_and(&mut res as *mut mlx_array, a.as_ptr(), b.as_ptr(), device.as_ptr());
    }

    // now mark result as initialized and return
    let result = unsafe {
        Array::from_ptr(res)
    };

    Ok(result)
}

// Optimized causal mask creation (if MLX supports vectorized ops)
fn create_causal_mask(batch_size: i32, seq_len: i32) -> Result<Array> {
    let device = Stream::default();
    // Use a more efficient method than nested loops if possible
    let causal_mask = arange(0, seq_len)?
        .unsqueeze(1)?  // (seq_len, 1)
        .tile(&[1, seq_len])?
        .ge(&mlx_arange(0, seq_len)?)?;  // (seq_len, seq_len)

    let causal_mask = broadcast_to(&causal_mask, &[batch_size, 1, seq_len, seq_len])?;
    Ok(causal_mask)
}

pub fn create_final_mask_internal(
    attention_mask: &Array,
    batch_size: i32,
    seq_len: i32,
) -> Result<Array> {
    // Inputs: (batch_size, seq_len) -> reshape once
    let attn_mask = attention_mask.reshape(&[batch_size, 1, 1, seq_len])?;

    let q_mask = attn_mask.unsqueeze(2)?;   // (bsz, 1, 1, seq_len)
    let k_mask = attn_mask.unsqueeze(3)?;   // (bsz, 1, seq_len, 1)

    // Combine masks
    let causal_mask = create_causal_mask(batch_size, seq_len)?;
    bitwise_and_arrays(&causal_mask, &q_mask)?.bitwise_and(&k_mask)
}


fn create_causal_mask(batch_size: i32, seq_len: i32) -> Result<Array> {
    // 1. Create lower triangular mask of shape (seq_len, seq_len)
    let mut tril = Array::zeros::<bool>(&[seq_len, seq_len])?;
    for i in 0..seq_len {
        for j in 0..=i {
            tril.index_mut((i, j), Array::from_bool(true));  // or bool true if supported
        }
    }

    // 2. Expand to (batch_size, 1, seq_len, seq_len)
    let causal_mask = broadcast_to(&tril, &[batch_size, 1, seq_len, seq_len])?;

    Ok(causal_mask)
}

pub fn create_final_mask(attention_mask: &Array) -> Result<Array> {
    let batch_size = attention_mask.shape()[0];
    let seq_len = attention_mask.shape()[1];

    // (batch_size, seq_len) -> (batch_size, 1, 1, seq_len)
    let attn_mask = attention_mask.clone().reshape(&[batch_size, 1, 1, seq_len])?;

    // Q mask: (batch_size, 1, seq_len, 1)
    let q_mask = attn_mask.reshape(&[batch_size, 1, seq_len, 1])?;

    // K mask: (batch_size, 1, 1, seq_len)
    let k_mask = attn_mask; // already in (batch_size, 1, 1, seq_len)

    // Create causal mask: (batch_size, 1, seq_len, seq_len)
    let causal_mask = create_causal_mask(batch_size, seq_len)?;

    // Combine: causal_mask & q_mask & k_mask
    let tmp = bitwise_and_arrays(&causal_mask, &q_mask)?;
    let final_mask = bitwise_and_arrays(&tmp, &k_mask)?;

    Ok(final_mask)
}
*/
