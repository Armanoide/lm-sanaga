use crate::cache::k_v_cache::ArcCacheItem;
use crate::config::config_models::llama::LLaMAConfig;
use crate::error::{Error, Result};
use crate::mask::mask::AttentionMask;
use crate::model::weight::Tensor;
use crate::module::Module;
use crate::quantized::Quantize;
use crate::utils::maybe_quantized::MaybeQuantizedLinear;
use mlx_rs::Array;
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::{Linear, LinearBuilder, silu};
use mlx_rs::quantization::{MaybeQuantized, Quantizable};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct MLPLlama {
    gate_proj: MaybeQuantized<Linear>,
    down_proj: MaybeQuantized<Linear>,
    up_proj: MaybeQuantized<Linear>,
}

impl Quantize for MLPLlama {
    fn quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        self.gate_proj = self
            .gate_proj
            .clone()
            .try_into_quantized(group_size, bits)?;
        self.down_proj = self
            .down_proj
            .clone()
            .try_into_quantized(group_size, bits)?;
        self.up_proj = self.up_proj.clone().try_into_quantized(group_size, bits)?;
        Ok(())
    }
}

impl Module for MLPLlama {
    fn forward(
        &mut self,
        x: &Array,
        _: Option<&AttentionMask>,
        _: Option<ArcCacheItem>,
    ) -> Result<Array> {
        // Apply gate projection and activation
        let gated = silu(self.gate_proj.forward(x)?)?;
        // Apply up projection
        let up = self.up_proj.forward(x)?;
        // Element-wise multiply
        let multiplied = gated * up;
        Ok(self.down_proj.forward(&multiplied)?)
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        if let Some(layer_without_suffix) = name.splitn(5, '.').nth(4) {
            match layer_without_suffix {
                "gate_proj.weight" => self.gate_proj.update_weight(&tensor.data),
                "down_proj.weight" => self.down_proj.update_weight(&tensor.data),
                "up_proj.weight" => self.up_proj.update_weight(&tensor.data),

                "gate_proj.scales" => self.gate_proj.update_scales(&tensor.data),
                "down_proj.scales" => self.down_proj.update_scales(&tensor.data),
                "up_proj.scales" => self.up_proj.update_scales(&tensor.data),

                "gate_proj.biases" => self.gate_proj.update_biases(&tensor.data),
                "down_proj.biases" => self.down_proj.update_biases(&tensor.data),
                "up_proj.biases" => self.up_proj.update_biases(&tensor.data),

                _ => return Err(Error::UnsupportedWeight(name.to_string())),
            }
        }
        Ok(())
    }
}

impl MLPLlama {
    pub fn new(config: Rc<LLaMAConfig>) -> Result<Self> {
        let dim = config.hidden_size;
        let hidden_dim = config.intermediate_size;
        let mlp_bias = config.mlp_bias;

        let gate_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: dim,
                output_dims: hidden_dim,
                bias: mlp_bias,
            }
            .build()?,
        );

        let down_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: hidden_dim,
                output_dims: dim,
                bias: mlp_bias,
            }
            .build()?,
        );

        let up_proj = MaybeQuantized::new(
            LinearBuilder {
                input_dims: dim,
                output_dims: hidden_dim,
                bias: mlp_bias,
            }
            .build()?,
        );

        Ok(MLPLlama {
            gate_proj,
            down_proj,
            up_proj,
        })
    }
}
