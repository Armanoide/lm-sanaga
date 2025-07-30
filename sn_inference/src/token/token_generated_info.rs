use crate::error::Result;
use crate::utils::mlx::get_peak_memory::get_peak_memory;

#[derive(Debug, Default)]
pub struct TokenGeneratedInfo {
    pub text: String,
    pub token: Vec<u32>,
    pub from_draft: bool,
    pub prompt_tokens: usize,
    pub peak_memory: usize,
    pub finish_reason: Option<String>,
    pub generation_tokens: usize,
}

impl TokenGeneratedInfo {
    pub fn set_token(&mut self, token: &[u32], generation_tokens: usize) {
        self.token = Vec::from(token);
        self.generation_tokens = generation_tokens;
    }

    pub fn set_text(&mut self, text: String) {
        print!("{}", text);
        self.text = text;
    }

    pub fn end(&mut self, with_reason: Option<String>) -> Result<()> {
        self.finish_reason = with_reason;
        self.peak_memory = get_peak_memory()?;

        Ok(())
    }
}
