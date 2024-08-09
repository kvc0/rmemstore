use bytes::Bytes;

use crate::types::{MemstoreItem, MemstoreWeigher};

pub struct RMemstoreServer {
    // cache: moka::sync::SegmentedCache<Bytes, MemstoreItem, ahash::RandomState>,
    cache: k_cache::SegmentedCache<Bytes, MemstoreItem, ahash::RandomState, MemstoreWeigher>,
}

// #[derive(Default)]
// struct Cache {
//     map: HashMap<Bytes, MemstoreItem, ahash::RandomState>,
//     /// fixme: probably I should try to implement the clock expiry thing
//     lru: VecDeque<Bytes>,
// }

// impl Cache {
//     fn put(&mut self, key: Bytes, value: MemstoreItem) {
//         if 2 << 20 < self.lru.len() {
//             let first = self.lru.pop_front().expect("it has a value");
//             self.map.remove(&first);
//         }
//         self.lru.push_back(key.clone());
//         self.map.insert(key, value);
//     }

//     fn get(&mut self, key: &[u8]) -> Option<MemstoreItem> {
//         self.map.get(key).cloned()
//     }
// }

impl RMemstoreServer {
    pub fn new() -> Self {
        // let cache = moka::sync::SegmentedCache::builder(8)
        //     .initial_capacity(2<<20)
        //     .weigher(MemstoreItem::weigher)
        //     .max_capacity(2<<20)
        //     .name("cache")
        //     .build_with_hasher(ahash::RandomState::new());
        Self {
            cache: k_cache::SegmentedCache::new(8, 2<<20),
        }
    }

    pub fn put(&self, key: Bytes, value: MemstoreItem) {
        self.cache.put(key, value)
    }

    pub fn get(&self, key: &[u8]) -> Option<MemstoreItem> {
        self.cache.get(key)
    }
}
