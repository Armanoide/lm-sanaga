use serde::Deserialize;
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ConfigTokenizerCustom {
    pub chat_template: String,
}

impl ConfigTokenizerCustom {}
