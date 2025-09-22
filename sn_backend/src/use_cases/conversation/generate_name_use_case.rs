use crate::error::Result;
use std::sync::{Arc, RwLock};

use sn_core::{types::conversation::Conversation, utils::rw_lock::RwLockExt};
use sn_inference::runner::Runner;

use crate::domain::conversation::aggregate::ConversationAggregate;

pub struct GenerateNameUseCase {
    runner: Arc<RwLock<Runner>>,
}

impl GenerateNameUseCase {
    pub fn new(runner: Arc<RwLock<Runner>>) -> Self {
        GenerateNameUseCase { runner }
    }

    pub async fn generate(&self, model_id: Arc<str>, agg: ConversationAggregate) -> Result<String> {
        let message = agg.first_user_message();
        let guard = self.runner.read_lock("reading runner for generate_name")?;
        let conversation = Conversation::from_user_with_content(format!(
            "resume with with 4 words only: {}",
            message
        ));
        let generate_text_result = guard.generate_text(&model_id, &conversation, None, None)?;
        let name = generate_text_result
            .0
            .trim()
            .replace('\n', "")
            .replace('\r', "");
        Ok(name)
    }
}
