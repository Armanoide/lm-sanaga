use mlx_rs::Array;
use mlx_rs::error::Exception;
use mlx_rs::nn::{Embedding, Linear};
use mlx_rs::quantization::{MaybeQuantized};

pub trait MaybeQuantizedLinear {
    fn update_weight(&mut self, x: &Array);
    fn update_scales(&mut self, x: &Array);
    fn update_biases(&mut self, x: &Array);

}

pub trait MaybeQuantizedEmbedding {
    fn as_linear(&mut self, x: &Array) -> Result<Array, Exception>;
    fn update_weight(&mut self, x: &Array);
    fn update_scales(&mut self, x: &Array);
    fn update_biases(&mut self, x: &Array);
}

impl MaybeQuantizedEmbedding for MaybeQuantized<Embedding> {

    fn as_linear(&mut self, x: &Array) -> Result<Array, Exception> {
        self.as_linear(x)
    }

    fn update_weight(&mut self, x: &Array) {
        if let MaybeQuantized::Quantized(q) = self {
            q.inner.weight.value = x.to_owned();
        }
    }

    fn update_scales(&mut self, x: &Array) {
        if let MaybeQuantized::Quantized(q) = self {
         q.scales.value = x.to_owned();
        }
    }

    fn update_biases(&mut self, x: &Array) {
        if let MaybeQuantized::Quantized(q) = self {
            q.biases.value = x.to_owned();
        }
    }
}

impl MaybeQuantizedLinear for MaybeQuantized<Linear> {
    #[allow(clippy::duplicate)]
    fn update_weight(&mut self, x: &Array) {
        if let MaybeQuantized::Quantized(q) = self {
            q.inner.weight.value = x.to_owned();
        }
    }

    #[allow(clippy::duplicate)]
    fn update_scales(&mut self, x: &Array) {
        if let MaybeQuantized::Quantized(q) = self {
            q.scales.value = x.to_owned();
        }
    }

    #[allow(clippy::duplicate)]
    fn update_biases(&mut self, x: &Array) {
        if let MaybeQuantized::Quantized(q) = self {
            q.biases.value = x.to_owned();
        }
    }
}
