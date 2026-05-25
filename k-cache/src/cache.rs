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

/// Result of a [`Lifecycle::evaluate`] check on an existing entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryStatus {
    /// The entry should stay in the cache as long as there is room
    Retain,
    /// The entry is evictable
    Evict,
}

pub trait Lifecycle<K, V> {
    /// Called when an entry has been evicted from the cache. Receives ownership
    /// of the key and value so they can be forwarded or dropped.
    fn on_eviction(&mut self, _key: K, _value: V) {}

    /// Inspect an existing entry and decide whether it should remain in the
    /// cache. Called as the sieve hand passes over entries during [`Cache::put`].
    fn evaluate(&self, _key: &K, _value: &V) -> EntryStatus {
        EntryStatus::Retain
    }
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
        // Advance the hand a few states so the sieve always progresses for Lifecycle.
        self.walk_hand();

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
        let (full_key, entry) = self.map.get_key_value(key)?;
        // If the Lifecycle says this entry is stale, report a miss. The hand
        // will clean it up the next time it passes over this position.
        if self.lifecycle.evaluate(full_key, &entry.data) == EntryStatus::Evict {
            return None;
        }
        entry
            .visited
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Some(&entry.data)
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

    /// Clears the cache, via eviction.
    ///
    /// If you want to handle the evicted entries, use the Lifecycle trait.
    /// The order of eviction is unspecified.
    pub fn evict_all(&mut self) {
        for (k, v) in self.map.drain() {
            self.lifecycle.on_eviction(k, v.data);
        }
        self.sieve_pool.clear();
        self.sieve_hand = 0;
        self.weight = 0;
    }

    /// Advance the sieve hand a small amount, [`Lifecycle::evaluate`]ing each.
    /// Entries whose evaluation returns [`EntryStatus::Evict`] are drained into
    /// `on_eviction` per usual.
    fn walk_hand(&mut self) {
        for _ in 0..3 {
            if self.sieve_pool.is_empty() {
                return;
            }
            let status = {
                let key = &self.sieve_pool[self.sieve_hand].data;
                match self.map.get(key) {
                    Some(entry) => self.lifecycle.evaluate(key, &entry.data),
                    None => EntryStatus::Evict,
                }
            };
            match status {
                EntryStatus::Evict => {
                    let sieve_key_entry = self
                        .sieve_pool
                        .swap_remove_back(self.sieve_hand)
                        .expect("the index must be present");
                    if let Some(removed_value) = self.remove(&sieve_key_entry.data) {
                        self.lifecycle
                            .on_eviction(sieve_key_entry.data, removed_value);
                    } else {
                        log::debug!("garbage collecting sieve entry at {}", self.sieve_hand);
                    }
                    if self.sieve_pool.is_empty() {
                        self.sieve_hand = 0;
                        return;
                    }
                    if self.sieve_hand >= self.sieve_pool.len() {
                        self.sieve_hand = 0;
                    }
                }
                EntryStatus::Retain => {
                    self.sieve_hand = (self.sieve_hand + 1) % self.sieve_pool.len();
                }
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
                // I don't care if the policy says to keep this value, I need the space. So out
                // of the cache it goes.
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
