use crate::error::Error;
use tokenizers::Encoding;

pub struct SimpleEncoding {
    ids: Vec<u32>,
    attention_mask: Vec<u32>,
}

impl SimpleEncoding {
    pub fn get_ids(&self) -> &[u32] {
        &self.ids
    }

    pub fn get_attention_mask(&self) -> &[u32] {
        &self.attention_mask
    }
}

pub fn pad_encode_batch(
    encodings: &[Encoding],
    padding_id: u32,
) -> crate::error::Result<Vec<SimpleEncoding>> {
    // Find max length
    let max_len = encodings.iter().map(|e| e.len()).max();
    let max_len = max_len.ok_or(Error::PaddingFailedFindMax)?;

    let mut new_encodings: Vec<SimpleEncoding> = Vec::new();

    // Pad each sequence
    for encoding in encodings {
        let mut ids = Vec::from(encoding.get_ids());
        ids.resize(max_len, padding_id);
        let mut attention_mask = Vec::from(encoding.get_attention_mask());
        attention_mask.resize(max_len, 0);
        new_encodings.push(SimpleEncoding {
            ids,
            attention_mask,
        });
    }

    Ok(new_encodings)
}
