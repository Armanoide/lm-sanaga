use crate::model::model::Model;
use std::sync::{Arc, RwLock};
use crate::cache::k_v_cache::k_v_cache::{ArcCacheList, KVCache};
use crate::model::model_runtime::ModelRuntime;
use crate::error::Result;

/*pub fn create_prompt_cache<T: Model + ?Sized>(model: &T) -> ArcCacheList {
    let n_layer = model.get_num_layer();
    let default_cache: Vec<Arc<RwLock<KVCache>>> = (0..n_layer)
        .map(|_| Arc::new(RwLock::new(KVCache::default())))
        .collect();
    Arc::new(RwLock::new(default_cache))
}*/

pub fn create_cache_from_model_runtime(model_runtime: Arc<ModelRuntime>) -> Result<ArcCacheList> {
    let n_layer = model_runtime.get_num_layer()?;
    let default_cache: Vec<Arc<RwLock<KVCache>>> = (0..n_layer)
        .map(|_| Arc::new(RwLock::new(KVCache::default())))
        .collect();
    Ok(Arc::new(RwLock::new(default_cache)))
}
