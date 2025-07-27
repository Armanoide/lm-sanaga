use crate::config::config_models::llama::LLaMAConfig;
use serde::de;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum ConfigModel {
    LLaMA(Rc<LLaMAConfig>),
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
            other => Err(de::Error::unknown_variant(other, &["llama"])),
        }
    }
}
