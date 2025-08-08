use crate::config::config_models::llama::LLaMAConfig;
use crate::config::config_models::qwen3::Qwen3Config;
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum ConfigModel {
    LLaMA(Rc<LLaMAConfig>),
    Qwen3(Rc<Qwen3Config>),
}

pub trait ConfigModelCommon {
    fn get_name(&self) -> String;
}

impl<'de> Deserialize<'de> for ConfigModel {
    fn deserialize<D>(deserializer: D) -> Result<ConfigModel, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Map<String, Value> = Deserialize::deserialize(deserializer)?;

        let model_type = value
            .get("model_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| de::Error::missing_field("model_type"))?;

        match model_type {
            "llama" => {
                let llama_value = Value::Object(value);
                let llama_config: LLaMAConfig =
                    serde_json::from_value(llama_value).map_err(de::Error::custom)?;
                Ok(ConfigModel::LLaMA(Rc::new(llama_config)))
            }
            "qwen3" => {
                let qwen3_value = Value::Object(value);
                let qwen3_config: Qwen3Config =
                    serde_json::from_value(qwen3_value).map_err(de::Error::custom)?;
                Ok(ConfigModel::Qwen3(Rc::new(qwen3_config)))
            }
            other => Err(de::Error::unknown_variant(other, &["qwen3"])),
        }
    }
}

impl Serialize for ConfigModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ConfigModel::LLaMA(config) => config.serialize(serializer),
            ConfigModel::Qwen3(config) => config.serialize(serializer),
        }
    }
}
