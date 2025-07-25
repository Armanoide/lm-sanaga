use crate::config::config_model::ConfigModel;
use crate::model::models::llama::rope::RopeLlama;
use crate::error::{Result, Error};

pub enum RopeModelType {
    LLaMA,
}
pub fn initialize_rope(dims: i32, base: f32, traditional: bool, config_model: ConfigModel) -> Result<RopeLlama> {
    match config_model {
        ConfigModel::LLaMA(llama_config) => {
            if let Some(rope_scaling_config) = &llama_config.rope_scaling {
                RopeLlama::new(dims, base, traditional, rope_scaling_config)
            } else {
                Err(Error::RopeConfigMissing)
            }
        }
    }

    /*
    if rope_type in ["default", "linear"]:
        scale = 1 / scaling_config["factor"] if rope_type == "linear" else 1.0
    return nn.RoPE(dims, traditional=traditional, base=base, scale=scale)
    */
}
