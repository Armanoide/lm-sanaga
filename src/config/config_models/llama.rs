use serde::Deserialize;
use crate::config::config_model::ConfigModelCommon;

#[allow(unused_variables)]
#[derive(Debug, Deserialize, Clone)]
pub struct LLaMAConfig {
    pub architectures: Vec<String>,
    pub attention_bias: bool,
    pub attention_dropout: f32,
    pub bos_token_id: i32,
    pub eos_token_id: Vec<i32>,
    pub hidden_act: String,
    pub hidden_size: i32,
    pub initializer_range: f32,
    pub intermediate_size: i32,
    pub max_position_embeddings: i32,
    pub mlp_bias: bool,
    pub model_type: String,
    pub num_attention_heads: i32,
    pub num_hidden_layers: i32,
    pub num_key_value_heads: i32,
    pub pretraining_tp: i32,
    pub quantization: Option<LLaMAQuantizationConfig>,
    pub quantization_config: Option<LLaMAQuantizationConfig>,
    pub rms_norm_eps: f32,
    pub rope_scaling: Option<LLaMARopeScalingConfig>,
    pub rope_theta: f32,
    pub tie_word_embeddings: bool,
    pub torch_dtype: String,
    pub transformers_version: String,
    pub use_cache: bool,
    pub vocab_size: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LLaMAQuantizationConfig {
    pub group_size: i32,
    pub bits: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LLaMARopeScalingConfig {
    pub factor: f32,
    pub low_freq_factor: f32,
    pub high_freq_factor: f32,
    pub original_max_position_embeddings: i32,
    pub rope_type: String,
}

impl ConfigModelCommon for LLaMAConfig  {
    fn get_name(&self,) -> String {
        let model_type = if let Some(rope_scaling) = &self.rope_scaling {
            match rope_scaling.rope_type.as_str() {
                "llama3" => "llama-3.1",
                other => other
            }
        } else {
            "unknown"
        };

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

        format!("models-{}-{}-{}{}", model_type, size, "Instruct", format!("-{}", quant))
    }
}
