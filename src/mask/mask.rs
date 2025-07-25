use mlx_rs::Array;
use mlx_rs::fast::ScaledDotProductAttentionMask;

#[derive(Debug)]
pub enum AttentionMask {
    Causal,
    MaskArray(Array),
}


impl AttentionMask {
    pub fn to_scaled_mask_opt(&self) -> Option<ScaledDotProductAttentionMask> {
        match self {
            AttentionMask::Causal => Some(ScaledDotProductAttentionMask::Causal),
            AttentionMask::MaskArray(arr) => Some(ScaledDotProductAttentionMask::from(arr)),
        }
    }
}
