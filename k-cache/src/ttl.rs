use std::time::Duration;

use crate::cache::{DefaultLifecycle, EntryStatus, Lifecycle};

/// Implemented by cache values that know how long they have left to live.
///
/// [`TtlLifecycle`] uses this to decide whether an entry is still fresh during
/// [`Lifecycle::evaluate`]. A value that returns [`Duration::ZERO`] is treated
/// as expired and will be evicted the next time the sieve hand reaches it.
///
/// Note that you may be asked multiple times about the same key even if you say
/// zero.
pub trait Ttl {
    /// How much time the value has left. `Duration::ZERO` means expired.
    fn remaining(&self) -> Duration;
}

/// Lifecycle decorator that evicts entries by [`Ttl::remaining`].
///
/// `on_eviction` is forwarded to the inner Lifecycle.
/// `evaluate` is forwarded only when the value is still fresh.
///
/// Wrap any other Lifecycle to combine TTL with custom eviction policies.
#[derive(Debug, Clone, Copy, Default)]
pub struct TtlLifecycle<L = DefaultLifecycle> {
    inner: L,
}

impl TtlLifecycle<DefaultLifecycle> {
    pub fn new() -> Self {
        Self {
            inner: DefaultLifecycle,
        }
    }
}

impl<L> TtlLifecycle<L> {
    pub fn with_inner(inner: L) -> Self {
        Self { inner }
    }
}

impl<K, V, L> Lifecycle<K, V> for TtlLifecycle<L>
where
    V: Ttl,
    L: Lifecycle<K, V>,
{
    fn on_eviction(&mut self, key: K, value: V) {
        self.inner.on_eviction(key, value);
    }

    fn evaluate(&self, key: &K, value: &V) -> EntryStatus {
        if value.remaining() == Duration::ZERO {
            EntryStatus::Evict
        } else {
            self.inner.evaluate(key, value)
        }
    }
}

#[cfg(test)]
mod test {
    use std::hash::RandomState;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use super::*;
    use crate::Cache;

    #[derive(Debug, Clone)]
    struct Expiring<V> {
        value: V,
        expires_at: Instant,
    }

    impl<V> Ttl for Expiring<V> {
        fn remaining(&self) -> Duration {
            self.expires_at.saturating_duration_since(Instant::now())
        }
    }

    #[test]
    fn fresh_entries_are_returned() {
        let mut cache: Cache<String, Expiring<String>, RandomState, crate::One, TtlLifecycle> =
            Cache::new_with_lifecycle(RandomState::new(), 100, TtlLifecycle::new());
        cache.put(
            "k".to_string(),
            Expiring {
                value: "v".to_string(),
                expires_at: Instant::now() + Duration::from_secs(60),
            },
        );
        assert_eq!(cache.get("k").map(|e| e.value.as_str()), Some("v"));
    }

    #[test]
    fn expired_entries_report_a_miss_on_get() {
        let mut cache: Cache<String, Expiring<String>, RandomState, crate::One, TtlLifecycle> =
            Cache::new_with_lifecycle(RandomState::new(), 100, TtlLifecycle::new());
        cache.put(
            "k".to_string(),
            Expiring {
                value: "v".to_string(),
                expires_at: Instant::now(),
            },
        );
        // already at/past expiry
        assert!(cache.get("k").is_none());
    }

    #[test]
    fn expired_entries_are_evicted_when_the_hand_reaches_them() {
        let evicted: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        #[derive(Clone)]
        struct Recorder(Arc<Mutex<Vec<String>>>);
        impl Lifecycle<String, Expiring<String>> for Recorder {
            fn on_eviction(&mut self, key: String, _value: Expiring<String>) {
                self.0.lock().expect("lock poison").push(key);
            }
        }

        let lifecycle = TtlLifecycle::with_inner(Recorder(evicted.clone()));
        let mut cache: Cache<
            String,
            Expiring<String>,
            RandomState,
            crate::One,
            TtlLifecycle<Recorder>,
        > = Cache::new_with_lifecycle(RandomState::new(), 100, lifecycle);

        cache.put(
            "stale".to_string(),
            Expiring {
                value: "x".to_string(),
                expires_at: Instant::now(),
            },
        );
        // hand advances 2 states per put; one extra put is enough to land on
        // the lone stale entry and evict it via advance_hand.
        cache.put(
            "fresh".to_string(),
            Expiring {
                value: "y".to_string(),
                expires_at: Instant::now() + Duration::from_secs(60),
            },
        );

        assert_eq!(
            evicted.lock().expect("lock poison").as_slice(),
            &["stale".to_string()]
        );
        assert!(cache.get("stale").is_none());
        assert!(cache.get("fresh").is_some());
    }
}
