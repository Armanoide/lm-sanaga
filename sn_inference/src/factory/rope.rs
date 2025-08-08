use crate::config::config_model::ConfigModel;
use crate::error::{Error, Result};
use crate::model::models::llama::rope::RopeLlama;
use crate::model::models::qwen3::rope::RopeQwen3;

pub enum RopeModelType {
    LLaMA(RopeLlama),
    Qwen3(RopeQwen3),
}
pub fn initialize_rope(
    dims: i32,
    base: f32,
    traditional: bool,
    config_model: ConfigModel,
) -> Result<RopeModelType> {
    match config_model {
        ConfigModel::LLaMA(llama_config) => {
            if let Some(rope_scaling_config) = &llama_config.rope_scaling {
                Ok(RopeModelType::LLaMA(RopeLlama::new(
                    dims,
                    base,
                    traditional,
                    rope_scaling_config,
                )?))
            } else {
                Err(Error::RopeConfigMissing)
            }
        }
        ConfigModel::Qwen3(qwen3_config) => {
            Ok(RopeModelType::Qwen3(RopeQwen3::new(
                dims,
                base,
                traditional,
            )?))
            /*if let Some(_) = &qwen3_config.rope_scaling {
            } else {
                // should never happen, but just in case
                // but //todo : make a default rope
                Err(Error::RopeConfigMissing)
            }*/
        }
    }
}
