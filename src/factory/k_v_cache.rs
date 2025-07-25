use std::sync::{Arc, RwLock};
use crate::cache::k_v_cache::{KVCache, SNCacheList};
use crate::model::model::Model;

pub fn create_prompt_cache<T: Model + ?Sized>(model: &T) -> SNCacheList {
    let n_layer = model.get_num_layer();
    let default_cache: Vec<Arc<RwLock<KVCache>>> = (0..n_layer)
        .map(|_| Arc::new(RwLock::new(KVCache::default())))
        .collect();
     Arc::new(RwLock::new(default_cache))
}
