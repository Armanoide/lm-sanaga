use crate::types::message_stats::MessageStats;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct TextGeneratedMetadataResponseSSE {
    pub prompt_tps: Option<f64>,
    pub generation_tps: Option<f64>,
    pub conversation_id: Option<i32>,
}

pub trait IntoMessageStat {
    fn into_message_stat(self, conversation_id: Option<i32>) -> TextGeneratedMetadataResponseSSE;
}

impl IntoMessageStat for MessageStats {
    fn into_message_stat(self, conversation_id: Option<i32>) -> TextGeneratedMetadataResponseSSE {
        TextGeneratedMetadataResponseSSE {
            prompt_tps: Some(self.prompt_tps),
            generation_tps: Some(self.generation_tps),
            conversation_id,
        }
    }
}
