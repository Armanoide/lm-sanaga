use crate::chat_template::chat_template::render_chat_template;
use crate::config::config::Config;
use crate::config::config_model::ConfigModel;
use crate::error::{Error, Result};
use crate::token::token_generated_info::TokenGeneratedInfo;
use minijinja::Environment;
use rayon::prelude::*;
use sn_core::types::conversation::Conversation;
use std::collections::HashSet;
use std::rc::Rc;
use tokenizers::tokenizer::Tokenizer as HugTokenizer;
use tracing::debug;

#[derive(Debug)]
pub struct Tokenizer {
    tool: HugTokenizer,
    config: Rc<Config>,
}

impl Tokenizer {
    pub fn new(config: Rc<Config>) -> Result<Tokenizer> {
        debug!("loading config in {}", &config.tokenizer_path);
        let tool = HugTokenizer::from_file(&config.tokenizer_path)?;
        Ok(Tokenizer { tool, config })
    }

    pub fn get_chat_template(&self) -> String {
        // check string or dict ?
        self.config.tokenizer_custom.chat_template.clone()
    }

    pub fn apply_chat_template(
        &self,
        conversations: &Conversation,
    ) -> Result<(String, Option<Vec<(usize, usize)>>)> {
        let mut env = Environment::new();
        let mut chat_template = self.get_chat_template();
        let chat_template_name = "chat";

        {// Some Patch python code to minijinja template for rust
            // remove think from template & messages (used for Qwen3)
            chat_template = chat_template.replace(
                "message.content.split('</think>')[-1].lstrip('\\n')",
                "(message.content | split('</think>'))[-1] | trim",
            );
            chat_template = chat_template.replace(
                "message.content.startswith('<tool_response>')",
                "message.content[:15] == '<tool_response>'"
            );
            chat_template = chat_template.replace(
                "message.content.endswith('</tool_response>')",
                "message.content[-16:] == '</tool_response>'"
            );
        }

        env.add_template(chat_template_name, chat_template.as_str())?;
        render_chat_template(
            &conversations,
            None,
            None,
            &env,
            chat_template_name,
            false,
            false,
            Some(true),
        )
    }

    pub fn encode_prompt(&self, messages: Vec<String>) -> Result<Vec<u32>> {
        match self.tool.encode_batch(messages, true) {
            Ok(encoding) => Ok(encoding
                .par_iter()
                .flat_map(|e| e.get_ids().to_owned())
                .collect()),
            Err(e) => Err(Error::EncodingProcessingError(e)),
        }
    }

    pub fn decode_response(&self, ids: &Vec<u32>, skip_special_tokens: bool) -> String {
        self.tool.decode(ids, skip_special_tokens).unwrap()
    }

    pub fn decode_response_from_generated_token_info(
        &self,
        generated_token_info: &mut TokenGeneratedInfo,
        has_header_start: bool,
        has_header_end: bool,
    ) {
        let token_ids = &generated_token_info.original_token;

        let s = self.decode_response(token_ids, true);
        if !has_header_start || has_header_start && has_header_end {
            generated_token_info.set_text(s);
        }
    }

    pub fn header_token_ids(&self) -> HashSet<u32> {
        self.tool
            .get_added_tokens_decoder()
            .par_iter()
            .filter_map(|(id, ad)| {
                if ad.content.contains("<|start_header_id|>")
                    || ad.content.contains("<|end_header_id|>")
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn eot_ids(&self) -> HashSet<u32> {
        match self.config.model.as_ref() {
            ConfigModel::LLaMA(config) => config
                .eos_token_id
                .iter()
                .map(|i| i.clone() as u32)
                .collect(),
            ConfigModel::Qwen3(config) => HashSet::from([config.eos_token_id as u32]),
        }
    }
}
