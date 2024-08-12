use bytes::Bytes;

use crate::types::{MemstoreItem, MemstoreWeigher};

pub struct RMemstoreServer {
    cache: k_cache::SegmentedCache<Bytes, MemstoreItem, ahash::RandomState, MemstoreWeigher>,
}

impl RMemstoreServer {
    pub fn new(segments: usize, cache_bytes: usize) -> Self {
        Self {
            cache: k_cache::SegmentedCache::new(segments, cache_bytes),
        }
    }

    pub fn put(&self, key: Bytes, value: MemstoreItem) {
        self.cache.put(key, value)
    }

    pub fn get(&self, key: &[u8]) -> Option<MemstoreItem> {
        self.cache.get(key)
    }
}
