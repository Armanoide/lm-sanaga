use crate::error::Result;
use crate::model::model::{ForwardType, Model};
use crate::model::model_kind::ModelKind;
use mlx_rs::linalg::norm;
use mlx_rs::ops::indexing::IndexOp;
use mlx_rs::ops::stack;
use mlx_rs::{Array, maximum};
use sn_core::utils::rw_lock::RwLockExt;
use std::sync::{Arc, RwLock};

pub struct TokenEmbeddingGenerator {
    model: Arc<RwLock<ModelKind>>,
}

impl TokenEmbeddingGenerator {
    pub fn new(model: Arc<RwLock<ModelKind>>) -> Self {
        TokenEmbeddingGenerator { model }
    }

    pub fn generate(&self, input: &Array, attention_mask: &Array) -> Result<Array> {
        let context = "read model for forwarding";
        let token_embeddings = self.model.write_lock(context)?.forward_model(
            &input,
            None,
            None,
            &ForwardType::Embedding,
        )?;
        let token_embeddings = self.pooling(&token_embeddings, attention_mask)?;
        let token_embeddings = self.normalize(&token_embeddings)?;
        Ok(token_embeddings)
    }

    fn pooling(&self, hidden_states: &Array, attention_mask: &Array) -> Result<Array> {
        // Compute the index of the last valid token for each sequence
        let sequence_lengths = attention_mask.sum_axis(-1, true)? - 1;
        let batch_size = hidden_states.shape()[0];

        // Flatten sequence_lengths into a vector of indices
        let last_token_indices = sequence_lengths.flatten(0, -1)?.as_slice::<i64>().to_vec();
        let mut list = Vec::new();

        // Collect the hidden states of the last token for each sequence
        for i in 0..batch_size {
            let index_1 = i as i32;
            let index_2 = last_token_indices[i as usize] as i32;
            list.push(hidden_states.index((index_1, index_2, ..)));
        }

        Ok(stack(&list)?)
    }

    fn normalize(&self, hidden_states: &Array) -> Result<Array> {
        let norm = norm(hidden_states, 2.0, &[-1], true)?;
        let embeddings = hidden_states / maximum!(norm, Array::from_f32(1e-9_f32))?;
        embeddings.eval()?;
        Ok(embeddings)
    }
}
