use serde::Deserialize;
#[derive(Deserialize, Debug, Clone)]
#[derive(Default)]
pub struct ConfigTokenizerCustom {
    pub chat_template: String,
}

impl ConfigTokenizerCustom {
}