use crate::error::Result;
use crate::utils::mlx::get_peak_memory::get_peak_memory;

#[derive(Debug, Default)]
pub struct TokenGeneratedInfo {
    pub text: String,
    pub original_token: Vec<u32>,
    pub from_draft: bool,
    pub prompt_tokens: usize,
    pub peak_memory: usize,
    pub finish_reason: Option<String>,
    pub generation_tokens: usize,
}

impl TokenGeneratedInfo {
    pub fn set_token(&mut self, token: &[u32], generation_tokens: usize) {
        self.original_token = Vec::from(token);
        self.generation_tokens = generation_tokens;
    }

    /// Returns the only token in the array. Panics if the array is unexpectedly empty.
    /// This assumes `token` always has exactly one element.
    pub fn get_token(&self) -> &u32 {
        &self.original_token.first().expect("token should contain exactly one element")
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn end(&mut self, with_reason: Option<String>) -> Result<()> {
        self.finish_reason = with_reason;
        self.peak_memory = get_peak_memory()?;

        Ok(())
    }
}
