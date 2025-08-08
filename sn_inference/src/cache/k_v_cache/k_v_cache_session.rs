use crate::cache::k_v_cache::k_v_cache::ArcCacheList;

#[derive(Clone, Debug)]
pub struct KvCacheSession {
    pub cache: ArcCacheList,
    pub session_id: i32,
}

impl KvCacheSession {

}