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

struct SieveEntry<D> {
    data: D,
    visited: Arc<AtomicBool>,
}

pub struct Cache<K, V, S, W: Weigher<K, V> = One> {
    map: HashMap<K, SieveEntry<V>, S>,
    sieve_pool: VecDeque<SieveEntry<K>>,
    sieve_hand: usize,
    max_weight: usize,
    weight: usize,
    _phantom: PhantomData<W>,
}

impl<K, V, S, W> Cache<K, V, S, W>
where
    K: Eq + Hash + Clone,
    S: BuildHasher,
    W: Weigher<K, V>,
{
    pub fn new(hasher: S, max_weight: usize) -> Self {
        Self {
            map: HashMap::with_hasher(hasher),
            sieve_pool: VecDeque::new(),
            sieve_hand: 0,
            max_weight,
            weight: 0,
            _phantom: PhantomData,
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        let new_weight = self.make_room_for(&key, &value);
        self.weight += new_weight;
        let visited = Arc::new(AtomicBool::new(true));
        self.map.insert(
            key.clone(),
            SieveEntry {
                data: value,
                visited: visited.clone(),
            },
        );
        self.sieve_pool.push_back(SieveEntry {
            data: key.clone(),
            visited,
        });
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
                let sieve_entry = self
                    .sieve_pool
                    .swap_remove_back(self.sieve_hand)
                    .expect("the index must be present");
                match self.map.remove(&sieve_entry.data) {
                    Some(removed) => {
                        let removed_weight = W::weigh(key, &removed.data);
                        self.weight -= removed_weight;
                    }
                    None => {
                        log::debug!("garbage collecting sieve entry as {}", self.sieve_hand);
                    }
                }

                if self.sieve_hand == self.sieve_pool.len() {
                    self.sieve_hand = 0;
                }
            }
        }
        entry_weight
    }
}
