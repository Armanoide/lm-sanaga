use crate::config::config_model::ConfigModel;
use crate::config::config_tokenizer_custom::ConfigTokenizerCustom;
use crate::error::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use tracing::debug;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub model: Rc<ConfigModel>,
    pub tokenizer_custom: ConfigTokenizerCustom,
    // Paths:
    pub root_path: String,
    pub tokenizer_custom_path: String,
    pub tokenizer_path: String,
    pub tokenizer_vocab_path: String,
    pub special_tokens_path: String,
}

fn get_config_tokenizer_custom(
    tokenizer_custom_path: &str,
    root_path: &str,
) -> Result<ConfigTokenizerCustom> {
    let mut config_tokenizer_custom =
        Config::from_file::<ConfigTokenizerCustom>(&tokenizer_custom_path)?;

    if config_tokenizer_custom.chat_template.is_none() {
        // If the chat template is empty, we can try to read it from the root path
        let chat_template_path = format!("{}/chat_template.jinja", root_path);
        debug!("Reading chat template from {}", chat_template_path);
        let chat_template = fs::read_to_string(chat_template_path)?;
        config_tokenizer_custom.chat_template = Some(chat_template);
    }

    Ok(config_tokenizer_custom)
}

impl Config {
    pub fn new(path: &str) -> Result<Self> {
        let root_path = path.to_owned();
        let config_path = Path::new(&root_path)
            .join("config.json")
            .display()
            .to_string();
        let tokenizer_path = Path::new(&root_path)
            .join("tokenizer.json")
            .display()
            .to_string();
        let tokenizer_custom_path = Path::new(&root_path)
            .join("tokenizer_config.json")
            .display()
            .to_string();
        let tokenizer_vocab_path = Path::new(&root_path)
            .join("vocab.json")
            .display()
            .to_string();
        let special_tokens_path = Path::new(&root_path)
            .join("special_tokens_map.json")
            .display()
            .to_string();

        let config_model = Config::from_file::<ConfigModel>(&config_path)?;
        let config_tokenizer_custom =
            get_config_tokenizer_custom(&tokenizer_custom_path, &root_path)?;
        Ok(Config {
            model: Rc::new(config_model),
            tokenizer_custom: config_tokenizer_custom,
            root_path,
            tokenizer_custom_path,
            tokenizer_path,
            tokenizer_vocab_path,
            special_tokens_path,
        })
    }

    fn from_file<T: DeserializeOwned>(path: &str) -> Result<T> {
        let data = fs::read_to_string(&path)?;
        let config: T = serde_json::from_str(&data)?;
        Ok(config)
    }
}
