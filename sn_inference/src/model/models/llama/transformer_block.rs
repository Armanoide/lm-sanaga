use crate::cache::k_v_cache::k_v_cache::ArcCacheItem;
use crate::config::config_models::llama::LLaMAConfig;
use crate::default_forward_transformer_block;
use crate::error::{Error, Result};
use crate::mask::mask::AttentionMask;
use crate::model::models::llama::attention::AttentionLlama;
use crate::model::models::llama::mlp::MLPLlama;
use crate::model::weight::Tensor;
use crate::module::Module;
use crate::quantized::Quantize;
use crate::utils::rms_norm::NormExt;
use mlx_rs::Array;
use mlx_rs::builder::Builder;
use mlx_rs::module::Module as MLXModule;
use mlx_rs::nn::{RmsNorm, RmsNormBuilder};
use std::rc::Rc;
#[derive(Debug, Clone)]
pub struct TransformerBlockLlama {
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
    fn forward(
        &mut self,
        x: &Array,
        mask: Option<&AttentionMask>,
        cache: Option<ArcCacheItem>,
    ) -> Result<Array> {
        default_forward_transformer_block!(self, x, mask, cache)
    }

    fn set_weight(&mut self, name: &str, sub_name: &str, tensor: &Tensor) -> Result<()> {
        match sub_name {
            "post_attention_layernorm.weight" => {
                return Ok(self.post_attention_layernorm.update_weight(&tensor.data));
            }
            "input_layernorm.weight" => {
                return Ok(self.input_layernorm.update_weight(&tensor.data));
            }
            _ => {
                if let Some(base_sub_name) = sub_name.split(".").next() {
                    let exclude_part = format!("{}.", base_sub_name);
                    if let Some(sub_name) = name.split(exclude_part.as_str()).nth(1) {
                        return match base_sub_name {
                            "mlp" => Ok(self.mlp.set_weight(name, sub_name, tensor)?),
                            "self_attn" => Ok(self.self_attn.set_weight(name, sub_name, tensor)?),
                            _ => Err(Error::UnsupportedWeight(name.to_string())),
                        };
                    }
                }
            }
        }
        Err(Error::UnsupportedWeight(name.to_string()))
    }
}

impl TransformerBlockLlama {
    pub fn new(llama_config: Rc<LLaMAConfig>) -> Result<TransformerBlockLlama> {
        let self_attn = AttentionLlama::new(llama_config.clone())?;
        let mlp = MLPLlama::new(llama_config.clone())?;

        let input_layernorm = RmsNormBuilder {
            dimensions: llama_config.hidden_size,
            eps: llama_config.rms_norm_eps,
        }
        .build()
        .map_err(|e| Error::ExceptionMLX(e))?;
        let post_attention_layernorm = RmsNormBuilder {
            dimensions: llama_config.hidden_size,
            eps: llama_config.rms_norm_eps,
        }
        .build()
        .map_err(|e| Error::ExceptionMLX(e))?;

        Ok(TransformerBlockLlama {
            self_attn,
            mlp,
            input_layernorm,
            post_attention_layernorm,
        })
    }
}
