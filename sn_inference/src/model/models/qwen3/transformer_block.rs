use crate::error::{Error, Result};
use crate::mask::mask::AttentionMask;
use crate::model::weight::Tensor;
use crate::module::Module;
use crate::quantized::Quantize;
use crate::utils::rms_norm::NormExt;
use mlx_rs::Array;
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::{RmsNorm, RmsNormBuilder};
use std::rc::Rc;
use crate::cache::k_v_cache::k_v_cache::ArcCacheItem;
use crate::config::config_models::qwen3::Qwen3Config;
use crate::model::models::qwen3::attention::AttentionQwen3;
use crate::model::models::qwen3::mlp::MLPQwen3;

#[derive(Debug, Clone)]
pub struct TransformerBlockQwen3 {
    hidden_size: i32,
    self_attn: AttentionQwen3,
    mlp: MLPQwen3,
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
}

impl Quantize for TransformerBlockQwen3 {
    fn quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        self.mlp.quantize(group_size, bits)?;
        self.self_attn.quantize(group_size, bits)?;
        Ok(())
    }
}

impl Module for TransformerBlockQwen3 {
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array> {
        let normed_input = self.input_layernorm.forward(x)?;
        let attn_output = self.self_attn.forward(&normed_input, mask, cache)?;
        let residual = x + attn_output;
        let normed_residual = self.post_attention_layernorm.forward(&residual)?;
        // No cache for MLP
        let mlp_output = self.mlp.forward(&normed_residual, mask, None)?;
        Ok(residual + mlp_output)
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        if let Some(layer_without_suffix) = name.splitn(4, '.').nth(3) {
            match layer_without_suffix {
                "post_attention_layernorm.weight" => {
                    return Ok(self.post_attention_layernorm.update_weight(&tensor.data))
                }
                "input_layernorm.weight" => return Ok(self.input_layernorm.update_weight(&tensor.data)),
                _ => {
                    if let Some(submodule) = layer_without_suffix.split(".").nth(0) {
                        return match submodule {
                            "mlp" => Ok(self.mlp.set_weight(name, tensor)?),
                            "self_attn" => Ok(self.self_attn.set_weight(name, tensor)?),
                            _ => {
                                Err(Error::UnsupportedWeight(name.to_string()))
                            }
                        }
                    }
                }
            }
        }
        Err(Error::UnsupportedWeight(name.to_string()))
    }
}

impl TransformerBlockQwen3 {
    pub fn new(qwen3_config: Rc<Qwen3Config>) -> Result<TransformerBlockQwen3> {
        let self_attn = AttentionQwen3::new(qwen3_config.clone())?;
        let mlp = MLPQwen3::new(qwen3_config.clone())?;

        let input_layernorm = RmsNormBuilder {
            dimensions: qwen3_config.hidden_size,
            eps: qwen3_config.rms_norm_eps,
        } 
        .build()
        .map_err(|e| Error::ExceptionMLX(e))?;
        let post_attention_layernorm = RmsNormBuilder {
            dimensions: qwen3_config.hidden_size,
            eps: qwen3_config.rms_norm_eps,
        }
        .build()
        .map_err(|e| Error::ExceptionMLX(e))?;

        Ok(TransformerBlockQwen3 {
            hidden_size: qwen3_config.num_attention_heads,
            self_attn,
            mlp,
            input_layernorm,
            post_attention_layernorm,
        })
    }
}
