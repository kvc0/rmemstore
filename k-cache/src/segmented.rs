use std::{borrow::Borrow, hash::BuildHasher};

use crate::{
    Cache, One, Weigher,
    cache::{DefaultLifecycle, Lifecycle},
};

#[derive(Debug)]
pub struct SegmentedCache<
    K,
    V,
    S: BuildHasher = std::hash::RandomState,
    W: Weigher<K, V> = One,
    L: Lifecycle<K, V> = DefaultLifecycle,
> {
    #[allow(clippy::type_complexity)]
    segments: Vec<k_lock::Mutex<Cache<K, V, S, W, L>>>,
    hasher: S,
}

impl<K, V, S, W, L> SegmentedCache<K, V, S, W, L>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
    S: BuildHasher + Default,
    W: Weigher<K, V> + Clone,
    L: Lifecycle<K, V> + Default,
{
    pub fn new(segments: usize, max_weight: usize) -> Self {
        let weight_per_segment = max_weight / segments;
        let segments = (0..segments)
            .map(|_| {
                k_lock::Mutex::new(Cache::new_with_lifecycle(
                    S::default(),
                    weight_per_segment,
                    L::default(),
                ))
            })
            .collect();
        Self {
            segments,
            hasher: S::default(),
        }
    }
}

impl<K, V, S, W, L> SegmentedCache<K, V, S, W, L>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
    S: BuildHasher + Default,
    W: Weigher<K, V> + Clone,
    L: Lifecycle<K, V> + Clone,
{
    pub fn new_with_lifecycle(segments: usize, max_weight: usize, lifecycle: L) -> Self {
        let weight_per_segment = max_weight / segments;
        let segments = (0..segments)
            .map(|_| {
                k_lock::Mutex::new(Cache::new_with_lifecycle(
                    S::default(),
                    weight_per_segment,
                    lifecycle.clone(),
                ))
            })
            .collect();
        Self {
            segments,
            hasher: S::default(),
        }
    }

    pub fn put(&self, key: K, value: V) {
        let slot = self.hasher.hash_one(&key) as usize % self.segments.len();
        self.segments[slot]
            .lock()
            .expect("mutex must not be poisoned")
            .put(key, value)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let slot = self.hasher.hash_one(key) as usize % self.segments.len();
        self.segments[slot]
            .lock()
            .expect("mutex must not be poisoned")
            .remove(key)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        let slot = self.hasher.hash_one(key) as usize % self.segments.len();
        self.segments[slot]
            .lock()
            .expect("mutex must not be poisoned")
            .get(key)
            .cloned()
    }
    /// Clears the cache, via eviction.
    ///
    /// If you want to handle the evicted entries, use the Lifecycle trait.
    /// The order of eviction is unspecified.
    ///
    /// This function does not guarantee that the cache is empty after calling it
    /// in the presence of concurrent writes. If you want to empty the cache,
    /// you need to stop writes first, and then call this function.
    /// If you just want to reset the cache, then you can call this function with
    /// concurrent writes. It's safe, it's just not going to be empty.
    ///
    /// It will block `1/segments` of the cache at a time for segment size time, so
    /// it's potentially fairly intrusive with large segments, especially with an
    /// expensive on_eviction Lifecycle if you have one.
    pub fn evict_all(&self) {
        for segment in &self.segments {
            segment
                .lock()
                .expect("mutex must not be poisoned")
                .evict_all();
        }
    }
}
