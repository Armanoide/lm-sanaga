use crate::cache::k_v_cache::{ArcCacheList, KVCache};
use crate::model::model::Model;
use std::sync::{Arc, RwLock};

pub fn create_prompt_cache<T: Model + ?Sized>(model: &T) -> ArcCacheList {
    let n_layer = model.get_num_layer();
    let default_cache: Vec<Arc<RwLock<KVCache>>> = (0..n_layer)
        .map(|_| Arc::new(RwLock::new(KVCache::default())))
        .collect();
    Arc::new(RwLock::new(default_cache))
}
