use crate::error::Result;
use mlx_rs::Array;
use mlx_rs::module::Param;
use mlx_rs::nn::{Embedding, Linear};
use mlx_rs::quantization::{MaybeQuantized, Quantizable};
use tracing::log::warn;

macro_rules! update_weight {
    ($self:ident, $x:expr) => {
        match $self {
            MaybeQuantized::Original(o) => {
                o.weight.value = $x.clone();
            }
            MaybeQuantized::Quantized(q) => q.inner.weight.value = $x.clone(),
        }
    };
}

macro_rules! update_scales {
    ($self:ident, $x:expr) => {
        match $self {
            MaybeQuantized::Original(_) => {
                warn!("update_scales called on original, should not happen");
            }
            MaybeQuantized::Quantized(q) => q.scales.value = $x.clone(),
        }
    };
}

macro_rules! update_biases {
    ($self:ident, $x:expr) => {
        match $self {
            MaybeQuantized::Original(_) => {
                warn!("update_biases called on original, should not happen");
            }
            MaybeQuantized::Quantized(q) => q.biases.value = $x.clone(),
        }
    };
}

pub trait MaybeQuantizedLinear {
    fn update_weight(&mut self, x: &Array);
    fn update_scales(&mut self, x: &Array);
    fn update_biases(&mut self, x: &Array);
}

pub trait MaybeQuantizedEmbedding {
    fn as_linear(&mut self, x: &Array) -> Result<Array>;
    fn update_weight(&mut self, x: &Array);
    fn update_scales(&mut self, x: &Array);
    fn update_biases(&mut self, x: &Array);
}

impl MaybeQuantizedEmbedding for MaybeQuantized<Embedding> {
    fn as_linear(&mut self, x: &Array) -> Result<Array> {
        match self {
            MaybeQuantized::Quantized(q) => Ok(q.as_linear(x)?),
            MaybeQuantized::Original(o) => Ok(o.as_linear(x)?),
        }
    }

    fn update_weight(&mut self, x: &Array) {
        update_weight!(self, x)
    }

    fn update_scales(&mut self, x: &Array) {
        update_scales!(self, x)
    }

    fn update_biases(&mut self, x: &Array) {
        update_biases!(self, x)
    }
}

impl MaybeQuantizedLinear for MaybeQuantized<Linear> {
    #[allow(clippy::duplicate)]
    fn update_weight(&mut self, x: &Array) {
        update_weight!(self, x);
    }

    #[allow(clippy::duplicate)]
    fn update_scales(&mut self, x: &Array) {
        update_scales!(self, x);
    }

    #[allow(clippy::duplicate)]
    fn update_biases(&mut self, x: &Array) {
        update_biases!(self, x);
    }
}

#[macro_export]
macro_rules! safe_quantize {
    ($self:ident, $group_size:expr, $bits:expr, $( $field:ident ),+ $(,)?) => {
        $(
            $self.$field.safe_quantize($group_size, $bits)?;
        )+
    };
}

pub trait QuantizableParam {
    fn safe_quantize(&mut self, group_size: i32, bits: i32) -> Result<()>;
}

impl QuantizableParam for MaybeQuantized<Embedding> {
    fn safe_quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        let dummy: MaybeQuantized<Embedding> = MaybeQuantized::new(Embedding {
            weight: Param::new(Array::from_bool(true)),
        });
        let old = std::mem::replace(self, dummy);
        *self = old.try_into_quantized(group_size, bits)?;
        Ok(())
    }
}

impl QuantizableParam for MaybeQuantized<Linear> {
    fn safe_quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        let dummy: MaybeQuantized<Linear> = MaybeQuantized::new(Linear {
            weight: Param::new(Array::from_bool(true)),
            bias: Param::new(None),
        });
        let old = std::mem::replace(self, dummy);
        *self = old.try_into_quantized(group_size, bits)?;
        Ok(())
    }
}
