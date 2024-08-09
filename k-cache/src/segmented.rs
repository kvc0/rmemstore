use std::{borrow::Borrow, hash::BuildHasher};

use crate::{Cache, One, Weigher};


pub struct SegmentedCache<K, V, S: BuildHasher = std::hash::RandomState, W: Weigher<K, V> = One> {
    segments: Vec<k_lock::Mutex<Cache<K, V, S, W>>>,
    hasher: S,
}

impl <K, V, S, W> SegmentedCache<K, V, S, W>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
    S: BuildHasher + Default,
    W: Weigher<K, V> + Clone,
{
    pub fn new(segments: usize, max_weight: usize) -> Self {
        let weight_per_segment = max_weight / segments;
        let segments = (0..segments).map(|_| k_lock::Mutex::new(Cache::new(S::default(), weight_per_segment))).collect();
        Self {
            segments,
            hasher: S::default(),
        }
    }

    pub fn put(&self, key: K, value: V) {
        let slot = self.hasher.hash_one(&key) as usize % self.segments.len();
        self.segments[slot].lock().expect("mutex must not be poisoned").put(key, value)
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        let slot = self.hasher.hash_one(&key) as usize % self.segments.len();
        self.segments[slot].lock().expect("mutex must not be poisoned").get(key).cloned()
    }
}
