#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct TextGeneratedMetadataResponseSSE {
    pub prompt_tps: Option<f64>,
    pub generation_tps: Option<f64>,
    pub conversation_id: i32,
}