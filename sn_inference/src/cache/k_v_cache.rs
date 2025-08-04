use mlx_rs::Array;
use mlx_rs::error::Exception;
use mlx_rs::ops::indexing::{IndexMutOp, IndexOp};
use mlx_rs::ops::{concatenate_axis, zeros_dtype};
use std::sync::{Arc, RwLock};

pub type ArcCacheItem = Arc<RwLock<KVCache>>;
pub type ArcCacheList = Arc<RwLock<Vec<ArcCacheItem>>>;
#[derive(Clone, Debug)]
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


    pub fn get_state(&self) -> (Array, Array) {
        if let (Some(keys), Some(values)) = (&self.keys, &self.values) {
            if keys.shape()[2] == self.offset {
                return (keys.clone(), values.clone());
            }
              return (
                  keys.index((.., ..self.offset, ..)),
                  values.index((.., ..self.offset, ..))
              )
        }
       (Array::from_int(0), Array::from_int(0))
    }

    #[allow(non_snake_case)]
    pub fn update_and_fetch_old(
        &mut self,
        keys: &Array,
        values: &Array,
    ) -> Result<(Array, Array), Exception> {
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

        keys_cache.index_mut((.., .., start..end, ..), keys);
        values_cache.index_mut((.., .., start..end, ..), values);
        let keys_out = keys_cache.index((.., .., 0..self.offset, ..));
        let values_out = values_cache.index((.., .., 0..self.offset, ..));
        Ok((keys_out, values_out))
    }

    #[allow(non_snake_case)]
    pub fn update_and_fetch(
        &mut self,
        keys: &Array,
        values: &Array,
    ) -> Result<(Array, Array), Exception> {
        let prev_offset = self.offset;
        let shape = keys.shape(); // [B, num_kv_heads, seq_len, head_dim]
        let new_seq_len = shape[2];

        // Resize logic: determine if we need to grow the cache
        let needs_resize = match &self.keys {
            Some(cached_keys) => (prev_offset + new_seq_len) > cached_keys.shape()[2],
            None => true,
        };

        if needs_resize {
            let B = shape[0];
            let num_kv_heads = shape[1];
            let k_head_dim = shape[3];
            let v_head_dim = values.shape()[3];

            // Calculate new capacity (rounded up to next step multiple)
            let total_needed = prev_offset + new_seq_len;
            let new_capacity = ((total_needed + self.step - 1) / self.step) * self.step;

            // New shapes
            let k_shape = [B, num_kv_heads, new_capacity, k_head_dim];
            let v_shape = [B, num_kv_heads, new_capacity, v_head_dim];

            let mut new_keys = zeros_dtype(&k_shape, keys.dtype())?;
            let mut new_values = zeros_dtype(&v_shape, values.dtype())?;

            // If existing cache exists, copy contents into new buffers
            if let Some(old_keys) = &self.keys {
                let old_len = old_keys.shape()[2].min(prev_offset);
                new_keys.index_mut(
                    (.., .., 0..old_len, ..),
                    &old_keys.index((.., .., 0..old_len, ..)),
                );
            }
            if let Some(old_values) = &self.values {
                let old_len = old_values.shape()[2].min(prev_offset);
                new_values.index_mut(
                    (.., .., 0..old_len, ..),
                    &old_values.index((.., .., 0..old_len, ..)),
                );
            }

            // Replace the old buffers with the new resized ones
            self.keys = Some(new_keys);
            self.values = Some(new_values);
        }

        // Unwrap safe because we just initialized them if they were None
        let keys_cache = self.keys.as_mut().unwrap();
        let values_cache = self.values.as_mut().unwrap();

        let start = prev_offset;
        let end = start + new_seq_len;

        // Write new keys/values into the cache
        keys_cache.index_mut((.., .., start..end, ..), keys);
        values_cache.index_mut((.., .., start..end, ..), values);

        // Update offset
        self.offset = end;

        // Return the cache slices up to the current offset
        let keys_out = keys_cache.index((.., .., 0..self.offset, ..));
        let values_out = values_cache.index((.., .., 0..self.offset, ..));

        Ok((keys_out, values_out))
    }
}

#[test]
fn test_kv_cache_update_and_fetch() {
    use mlx_rs::Dtype;
    use mlx_rs::{Array, zeros_dtype};

    let batch = 1;
    let heads = 2;
    let head_dim = 4;
    let seq_len = 3;
    let dtype = Dtype::Float16;

    let mut cache = KVCache::default();
    cache.step = 4; // For testing resize

    // Round 1: Add first key/value
    let keys1 = zeros_dtype(&[batch, heads, seq_len, head_dim], dtype).unwrap();
    let values1 = zeros_dtype(&[batch, heads, seq_len, head_dim], dtype).unwrap();
    let (out_k1, out_v1) = cache.update_and_fetch(&keys1, &values1).unwrap();

    assert_eq!(out_k1.shape(), &[batch, heads, seq_len, head_dim]);
    assert_eq!(out_v1.shape(), &[batch, heads, seq_len, head_dim]);

    // Round 2: Add another key/value
    let keys2 = zeros_dtype(&[batch, heads, seq_len, head_dim], dtype).unwrap();
    let values2 = zeros_dtype(&[batch, heads, seq_len, head_dim], dtype).unwrap();
    let (out_k2, out_v2) = cache.update_and_fetch(&keys2, &values2).unwrap();

    assert_eq!(out_k2.shape(), &[batch, heads, seq_len * 2, head_dim]);
    assert_eq!(out_v2.shape(), &[batch, heads, seq_len * 2, head_dim]);

    // Round 3: Add large input to trigger resize
    let big_seq = 6;
    let keys3 = zeros_dtype(&[batch, heads, big_seq, head_dim], dtype).unwrap();
    let values3 = zeros_dtype(&[batch, heads, big_seq, head_dim], dtype).unwrap();
    let (out_k3, out_v3) = cache.update_and_fetch(&keys3, &values3).unwrap();

    assert_eq!(
        out_k3.shape(),
        &[batch, heads, 2 * seq_len + big_seq, head_dim]
    );
    assert_eq!(
        out_v3.shape(),
        &[batch, heads, 2 * seq_len + big_seq, head_dim]
    );

    // Sanity: Final offset should match total seq_len
    assert_eq!(cache.offset, 2 * seq_len + big_seq);
}
