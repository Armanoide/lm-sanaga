use crate::config::config_model::ConfigModelCommon;
use crate::config::config_models::quantization_config::QuantizationConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Qwen3Config {
    pub architectures: Vec<String>,
    pub attention_bias: bool,
    pub attention_dropout: f64,

    // Only in emb
    pub bos_token_id: Option<i32>,

    pub eos_token_id: i32,
    pub head_dim: Option<i32>,
    pub hidden_act: String,
    pub hidden_size: i32,
    pub initializer_range: f64,
    pub intermediate_size: i32,
    pub max_position_embeddings: i32,
    pub max_window_layers: i32,
    pub model_type: String,
    pub num_attention_heads: i32,
    pub num_hidden_layers: i32,
    pub num_key_value_heads: i32,

    // Only in normal
    pub pad_token_id: Option<i32>,
    pub qkv_bias: Option<bool>,
    pub quantization: Option<QuantizationConfig>,
    pub quantization_config: Option<QuantizationConfig>,
    pub use_qk_norm: Option<bool>,

    pub rms_norm_eps: f32,
    pub rope_scaling: Option<Qwen3RopeScalingConfig>,
    pub rope_theta: f32,
    pub sliding_window: Option<serde_json::Value>,
    pub tie_word_embeddings: bool,
    pub torch_dtype: String,
    pub transformers_version: String,
    pub use_cache: bool,
    pub use_sliding_window: bool,
    pub vocab_size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Qwen3RopeScalingConfig {}

impl ConfigModelCommon for Qwen3Config {
    fn get_name(&self) -> String {
        let model_type = "Qwen3";

        let size = match (self.hidden_size, self.num_hidden_layers) {
            (4096, 32) => "8B",
            (5120, 40) => "13B",
            (6656, 60) => "70B",
            _ => "unknown-size",
        };

        let quant = if self.quantization.is_some() || self.quantization_config.is_some() {
            "4bit"
        } else {
            "fp16"
        };

        format!(
            "models-{}-{}-{}{}",
            model_type,
            size,
            "Instruct",
            format!("-{}", quant)
        )
    }
}
