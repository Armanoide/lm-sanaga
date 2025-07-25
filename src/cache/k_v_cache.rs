use std::sync::{Arc, RwLock};
use mlx_rs::Array;
use mlx_rs::error::Exception;
use mlx_rs::ops::{concatenate_axis, zeros_dtype};
use mlx_rs::ops::indexing::{IndexMutOp, IndexOp};

pub type SNCacheItem = Arc<RwLock<KVCache>>;
pub type SNCacheList = Arc<RwLock<Vec<SNCacheItem>>>;

#[derive(Clone,Debug)]
pub struct KVCache {
    pub keys: Option<Array>,
    pub values: Option<Array>,
    pub offset: i32,
    pub step: i32,
    pub max_size: Option<i32>,
}

impl KVCache {
    pub fn default() -> Self {
        Self {
            keys: None,
            values: None,
            offset: 0,
            step: 256,
            max_size: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn update_and_fetch(&mut self, keys: &Array, values: &Array) -> Result<(Array, Array), Exception> {
        let prev = self.offset;
        let shape = keys.shape();
        let new_seq_len = shape[2];
        let needs_resize = match &self.keys {
            Some(cached_keys) => (prev + keys.shape()[2]) > cached_keys.shape()[2],
            None => true,
        };

        if self.keys.is_none() || needs_resize {
            let prev = self.offset;
            let B = shape[0];
            let n_kv_heads = shape[1];
            let k_head_dim = shape[3];
            let v_head_dim = values.shape()[3];
            let n_steps = (self.step + shape[2] - 1) / self.step;
            let k_shape = [B, n_kv_heads, n_steps * self.step, k_head_dim];
            let v_shape = [B, n_kv_heads, n_steps * self.step, v_head_dim];
            let new_k = zeros_dtype(&k_shape, keys.dtype())?;
            let new_v = zeros_dtype(&v_shape, values.dtype())?;
            if let (Some(old_k), Some(old_v)) = (&self.keys.take(), &self.values.take()) {
                if prev % self.step != 0 {
                    self.keys = Some(old_k.index((.., 0..prev, ..)).clone());
                    self.values = Some(old_v.index((.., 0..prev, ..)).clone());
                }
                self.keys = Some(concatenate_axis(&[old_k, &new_k], 2)?);
                self.values = Some(concatenate_axis(&[old_v, &new_v], 2)?);
            } else {
                self.keys = Some(new_k);
                self.values = Some(new_v);
            }
        }

        self.offset += new_seq_len;
        // Write new keys and values into the cache
        let keys_cache = self.keys.as_mut().unwrap();
        let values_cache = self.values.as_mut().unwrap();

        let start = prev;
        let end = start + new_seq_len;

        keys_cache
            .index_mut((.., .., start..end, ..), keys);
        values_cache
            .index_mut((.., .., start..end, ..), values);
        let keys_out = keys_cache.index((.., .., 0..self.offset, ..));
        let values_out = values_cache.index((.., .., 0..self.offset, ..));
        Ok((keys_out, values_out))
    }
}