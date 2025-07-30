use crate::chat_template::chat_template::render_chat_template;
use crate::config::config::Config;
use crate::error::{Error, Result};
use crate::token::token_generated_info::TokenGeneratedInfo;
use tracing::debug;
use minijinja::Environment;
use rayon::prelude::*;
use sn_core::conversation::conversation::Conversation;
use std::rc::Rc;
use tokenizers::tokenizer::Tokenizer as HugTokenizer;

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

    pub fn get_chat_template(&self) -> &str {
        // check string or dict ?
        self.config.tokenizer_custom.chat_template.as_str()
    }

    pub fn apply_chat_template(
        &self,
        conversation: &Conversation,
    ) -> Result<(Vec<String>, Option<Vec<(usize, usize)>>)> {
        let mut env = Environment::new();
        let chat_template = self.get_chat_template();
        let chat_template_name = "chat";
        env.add_template(chat_template_name, chat_template)?;
        render_chat_template(
            conversation,
            None,
            None,
            &env,
            chat_template_name,
            false,
            false,
        )
    }

    pub fn encode_prompt(&self, messages: Vec<String>) -> Result<Vec<u32>> {
        match self.tool.encode_batch(messages, false) {
            Ok(encoding) => Ok(encoding
                .par_iter()
                .fold(
                    || Vec::<u32>::new(),
                    |a, b| [&a[..], &b.get_ids()[..]].concat(),
                )
                .reduce(|| Vec::<u32>::new(), |a, b| [&a[..], &b[..]].concat())),
            Err(e) => Err(Error::EncodingProcessingError(e)),
        }
    }

    pub fn decode_response(&self, ids: &Vec<u32>) -> String {
        self.tool.decode(ids, false).unwrap()
    }

    pub fn decode_response_from_generated_token_info(
        &self,
        generated_token_info: &mut TokenGeneratedInfo,
    ) {
        let token_ids = &generated_token_info.token;
        let s = self.decode_response(token_ids);
        generated_token_info.set_text(s);
    }

    pub fn eot_ids(&self) -> Vec<u32> {
        self.tool
            .get_added_tokens_decoder()
            .par_iter()
            .filter_map(|(id, ad)| {
                if ad.content.contains("<|eot_id|>")
                    || ad.content.contains("<|eos|>")
                    || ad.content.contains("<|endoftext|>")
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}
