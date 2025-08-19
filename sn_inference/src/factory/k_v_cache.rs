use crate::cache::k_v_cache::k_v_cache::{ArcCacheList, KVCache};
use crate::error::Result;
use crate::model::model_runtime::ModelRuntime;
use std::sync::{Arc, RwLock};

pub fn create_cache_from_model_runtime(model_runtime: Arc<ModelRuntime>) -> Result<ArcCacheList> {
    let n_layer = model_runtime.get_num_layer()?;
    let default_cache: Vec<Arc<RwLock<KVCache>>> = (0..n_layer)
        .map(|_| Arc::new(RwLock::new(KVCache::default())))
        .collect();
    Ok(Arc::new(RwLock::new(default_cache)))
}
