use mlx_rs::Array;
use crate::error::Error;
use crate::model::models::llama::attention::AttentionLlama;
use crate::model::models::llama::mlp::MLPLlama;
use crate::module::{Module};
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::{RmsNorm, RmsNormBuilder};
use crate::cache::k_v_cache::{SNCacheItem};
use crate::config::config_models::llama::LLaMAConfig;
use crate::mask::mask::AttentionMask;
use crate::model::weight::Tensor;
use crate::quantized::Quantize;
use crate::utils::rms_norm::NormExt;
use crate::error::Result;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct TransformerBlockLlama {
    hidden_size: i32,
    self_attn: AttentionLlama,
    mlp: MLPLlama,
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
}

impl Quantize for TransformerBlockLlama {
    fn quantize(&mut self, group_size: i32, bits: i32) -> Result<()> {
        self.mlp.quantize(group_size, bits)?;
        self.self_attn.quantize(group_size, bits)?;
        Ok(())
    }
}

impl Module for TransformerBlockLlama {
    fn forward(&mut self, x: &Array, mask: Option<&AttentionMask>, cache: Option<SNCacheItem>) -> Result<Array> {
        let mut r = self.self_attn.forward(&self.input_layernorm.forward(x)?, mask, cache.clone())?;
        let h = x + r;
        r = self.mlp.forward(&self.post_attention_layernorm.forward(&h)?, mask, cache.clone())?;
        Ok(h + r)
    }

    fn set_weight(&mut self, name: &str, tensor: &Tensor) -> Result<()> {
        // Split on '.' and skip the first 3 segments: "model", "layers", "<index>"
        if let Some(layer_without_suffix) = name.splitn(4, '.').nth(3) {
            match layer_without_suffix  {
                "post_attention_layernorm.weight" => self.post_attention_layernorm.update_weight(&tensor.data),
                "input_layernorm.weight" => self.input_layernorm.update_weight(&tensor.data),
                _ =>  {
                    if let Some(submodule) = layer_without_suffix.split(".").nth(0) {
                        match submodule {
                            "mlp" => self.mlp.set_weight(name, tensor)?,
                            "self_attn" => self.self_attn.set_weight(name, tensor)?,
                            _ => {
                                return Err(Error::UnsupportedWeight(name.to_string()));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl TransformerBlockLlama {
    pub fn new(llama_config: Rc<LLaMAConfig>) -> Result<TransformerBlockLlama> {
        let self_attn = AttentionLlama::new(llama_config.clone())?;
        let mlp = MLPLlama::new(llama_config.clone())?;

        let input_layernorm = RmsNormBuilder {
            dimensions: llama_config.hidden_size,
            eps:llama_config.rms_norm_eps
        }.build().map_err(|e| Error::ExceptionMLX(e))?;
        let post_attention_layernorm = RmsNormBuilder {
            dimensions: llama_config.hidden_size,
            eps:llama_config.rms_norm_eps
        }.build().map_err(|e| Error::ExceptionMLX(e))?;

        Ok(TransformerBlockLlama {
            hidden_size: llama_config.num_attention_heads,
            self_attn,
            mlp,
            input_layernorm,
            post_attention_layernorm
        })
    }
}
