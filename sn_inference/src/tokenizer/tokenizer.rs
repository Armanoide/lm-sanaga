use crate::config::config::Config;
use crate::config::config_model::ConfigModel;
use crate::error::{Error, Result};
use crate::token::token_generated_info::TokenGeneratedInfo;
use rayon::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;
use tokenizers::tokenizer::Tokenizer as HugTokenizer;
use tokenizers::{EncodeInput, Encoding, PaddingParams};
use tracing::debug;

#[derive(Debug)]
pub struct Tokenizer {
    tool: HugTokenizer,
    config: Rc<Config>,
}

fn add_padding_params(config: &Rc<Config>, tool: &mut HugTokenizer) -> Result<()> {
    if tool.get_padding().is_none()
        && let Some(padding) = &config.tokenizer_custom.pad_token
    {
        let added_tokens_decoder = tool.get_added_tokens_decoder();
        let token = added_tokens_decoder
            .par_iter()
            .find_any(|(_, ad)| ad.content == *padding);
        if let Some((id, _)) = token {
            let mut padding_params = PaddingParams::default();
            padding_params.pad_id = *id;
            tool.with_padding(Some(padding_params));
        } else {
            debug!("Padding token not found in tokenizer, using default padding.");
        }
    }
    Ok(())
}

impl Tokenizer {
    pub fn new(config: Rc<Config>) -> Result<Tokenizer> {
        debug!("loading config in {}", &config.tokenizer_path);
        let mut tool = HugTokenizer::from_file(&config.tokenizer_path)?;
        add_padding_params(&config, &mut tool)?;
        Ok(Tokenizer { tool, config })
    }
    pub fn get_pad_token_id(&self) -> Option<&PaddingParams> {
        self.tool.get_padding()
    }

    pub fn encode_batch<'s, E>(
        &self,
        input: Vec<E>,
        add_special_tokens: bool,
    ) -> Result<Vec<Encoding>>
    where
        E: Into<EncodeInput<'s>> + Send,
    {
        Ok(self.tool.encode_batch(input, add_special_tokens)?)
    }

    pub fn encode(&self, input: &str, add_special_tokens: bool) -> Result<Encoding> {
        Ok(self.tool.encode(input, add_special_tokens)?)
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
