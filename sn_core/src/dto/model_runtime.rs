use serde::Deserialize;
use serde::Serialize;
#[derive(Serialize, Default, Deserialize)]
pub struct ModelRuntimeDTO {
    pub name: String,
    pub id: String,
}

impl ModelRuntimeDTO {}
