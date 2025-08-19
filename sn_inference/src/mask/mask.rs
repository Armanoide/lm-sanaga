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
