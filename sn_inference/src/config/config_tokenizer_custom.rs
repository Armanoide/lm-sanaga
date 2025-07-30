use serde::{Deserialize, Serialize};
#[derive(Deserialize, Debug, Clone, Default, Serialize)]
pub struct ConfigTokenizerCustom {
    pub chat_template: String,
}

impl ConfigTokenizerCustom {}
