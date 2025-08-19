use serde::{Deserialize, Serialize};
const DEFAULT_CHAT_TEMPLATE: &str = "";
#[derive(Deserialize, Debug, Clone, Default, Serialize)]
pub struct ConfigTokenizerCustom {
    pub chat_template: Option<String>,
    pub pad_token: Option<String>,
}

impl ConfigTokenizerCustom {
    pub fn get_chat_template(&self) -> &str {
        if let Some(chat) = &self.chat_template {
            chat.as_ref()
        } else {
            DEFAULT_CHAT_TEMPLATE
        }
    }
}
