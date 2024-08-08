use std::collections::{HashMap, VecDeque};

use bytes::Bytes;

use crate::types::MemstoreItem;

pub struct RMemstoreServer {
    // cache: moka::sync::SegmentedCache<Bytes, MemstoreItem, ahash::RandomState>,
    random: ahash::RandomState,
    cache: Vec<k_lock::Mutex<Cache>>,
}

#[derive(Default)]
struct Cache {
    map: HashMap<Bytes, MemstoreItem, ahash::RandomState>,
    /// fixme: probably I should try to implement the clock expiry thing
    lru: VecDeque<Bytes>,
}

impl Cache {
    fn put(&mut self, key: Bytes, value: MemstoreItem) {
        if 2 << 20 < self.lru.len() {
            let first = self.lru.pop_front().expect("it has a value");
            self.map.remove(&first);
        }
        self.lru.push_back(key.clone());
        self.map.insert(key, value);
    }

    fn get(&mut self, key: &[u8]) -> Option<MemstoreItem> {
        self.map.get(key).cloned()
    }
}

impl RMemstoreServer {
    pub fn new() -> Self {
        // let cache = moka::sync::SegmentedCache::builder(8)
        //     .initial_capacity(2<<20)
        //     .weigher(MemstoreItem::weigher)
        //     .max_capacity(2<<20)
        //     .name("cache")
        //     .build_with_hasher(ahash::RandomState::new());
        Self {
            cache: Vec::from_iter((0..(2 << 2)).map(|_| k_lock::Mutex::new(Cache::default()))),
            random: ahash::RandomState::new(),
        }
    }

    pub fn put(&self, key: Bytes, value: MemstoreItem) {
        let position = (self.random.hash_one(&key) as usize) & (self.cache.len() - 1);
        self.cache[position]
            .lock()
            .expect("mutex works")
            .put(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<MemstoreItem> {
        let position = (self.random.hash_one(key) as usize) & (self.cache.len() - 1);
        self.cache[position].lock().expect("mutex works").get(key)
    }
}
