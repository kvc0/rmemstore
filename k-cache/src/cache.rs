use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::{
    borrow::Borrow,
    collections::{HashMap, VecDeque},
    hash::BuildHasher,
};

pub trait Weigher<K, V> {
    fn weigh(_k: &K, _v: &V) -> usize {
        1
    }
}

#[derive(Debug, Clone)]
pub struct One;
impl<K, V> Weigher<K, V> for One {}

pub trait Lifecycle<K, V> {
    fn on_eviction(&self, _key: K, _value: V) {}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultLifecycle;
impl<K, V> Lifecycle<K, V> for DefaultLifecycle {}

#[derive(Debug)]
struct SieveEntry<D> {
    data: D,
    visited: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct Cache<K, V, S, W: Weigher<K, V> = One, L: Lifecycle<K, V> = DefaultLifecycle> {
    map: HashMap<K, SieveEntry<V>, S>,
    sieve_pool: VecDeque<SieveEntry<K>>,
    sieve_hand: usize,
    max_weight: usize,
    weight: usize,
    lifecycle: L,
    _phantom: PhantomData<W>,
}

impl<K, V, S, W, L> Cache<K, V, S, W, L>
where
    K: Eq + Hash + Clone,
    S: BuildHasher,
    W: Weigher<K, V>,
    L: Lifecycle<K, V> + Default,
{
    pub fn new(hasher: S, max_weight: usize) -> Self {
        Self {
            map: HashMap::with_hasher(hasher),
            sieve_pool: VecDeque::new(),
            sieve_hand: 0,
            max_weight,
            weight: 0,
            lifecycle: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<K, V, S, W, L> Cache<K, V, S, W, L>
where
    K: Eq + Hash + Clone,
    S: BuildHasher,
    W: Weigher<K, V>,
    L: Lifecycle<K, V>,
{
    pub fn new_with_lifecycle(hasher: S, max_weight: usize, lifecycle: L) -> Self {
        Self {
            map: HashMap::with_hasher(hasher),
            sieve_pool: VecDeque::new(),
            sieve_hand: 0,
            max_weight,
            weight: 0,
            lifecycle,
            _phantom: PhantomData,
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        let new_entry_weight = self.make_room_for(&key, &value);
        self.weight += new_entry_weight;

        match self.map.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                let replaced_weight = W::weigh(&key, &occupied_entry.get().data);
                self.weight -= replaced_weight; // already added the new entry weight

                occupied_entry.get_mut().data = value;
                occupied_entry
                    .get_mut()
                    .visited
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                // it's still possible to get spurious evictions in here. I should replace the dirty atomicbool
                // with a tri-state int to track deletion separately from visitation. That could also allow some
                // stupid bitwise tricks to remember insertions and possibly eagerly sieve out unpopular, though
                // technically accessed, entries.
                let visited = Arc::new(AtomicBool::new(true));
                vacant_entry.insert(SieveEntry {
                    data: value,
                    visited: visited.clone(),
                });
                self.sieve_pool.push_back(SieveEntry { data: key, visited });
            }
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get(key) {
            Some(entry) => {
                entry
                    .visited
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                Some(&entry.data)
            }
            None => None,
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.map.remove(key) {
            Some(removed) => {
                // Note that this under-counts memory by accounting for the key size
                // up front. The key is still in the sieve list. Hopefully it will be
                // reclaimed soon. I could split the key and value weighers to do a
                // better accounting job.
                let removed_weight = W::weigh(key, &removed.data);
                match self.weight.checked_sub(removed_weight) {
                    Some(new_weight) => self.weight = new_weight,
                    None => {
                        log::error!("weight underflow");
                        self.weight = 0;
                    }
                };
                Some(removed.data)
            }
            None => {
                log::debug!("garbage collecting sieve entry as {}", self.sieve_hand);
                None
            }
        }
    }

    fn make_room_for(&mut self, key: &K, value: &V) -> usize {
        let entry_weight = W::weigh(key, value);
        while self.max_weight < self.weight + entry_weight {
            let sieve_entry = &mut self.sieve_pool[self.sieve_hand];
            let visited = sieve_entry
                .visited
                .swap(false, std::sync::atomic::Ordering::Relaxed);
            if visited {
                self.sieve_hand = (self.sieve_hand + 1) % self.sieve_pool.len();
            } else {
                let sieve_key_entry = self
                    .sieve_pool
                    .swap_remove_back(self.sieve_hand)
                    .expect("the index must be present");
                let removed = self.remove(&sieve_key_entry.data);
                if let Some(removed_value) = removed {
                    self.lifecycle
                        .on_eviction(sieve_key_entry.data, removed_value);
                } else {
                    // This can happen when the entry was already removed. It's not an eviction,
                    // but a clean up of the sieve list. The value was already released - we can
                    // simply drop the key.
                    log::debug!("garbage collecting sieve entry at {}", self.sieve_hand);
                }

                if self.sieve_hand == self.sieve_pool.len() {
                    self.sieve_hand = 0;
                }
            }
        }
        entry_weight
    }
}

#[cfg(test)]
mod test {
    use std::hash::RandomState;

    use super::*;

    #[test]
    fn test_put() {
        let mut cache: Cache<String, String, RandomState> = Cache::new(RandomState::new(), 100);
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some(&"value1".to_string()));
        cache.put("key1".to_string(), "value2".to_string());
        assert_eq!(cache.get("key1"), Some(&"value2".to_string()));
        assert_eq!(cache.weight, 1);
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.sieve_pool.len(), 1);
    }
}
