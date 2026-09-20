//! Maps to: CC `utils/memoize.ts#memoizeWithLRU`.
//! TTL and asynchronous TTL exports are outside this implementation slice.

use indexmap::IndexMap;
use std::sync::Mutex;

// Native carrier for JavaScript's synchronous memoized execution turn.
// A separate reentrant turn gate keeps concurrent OS threads from observing
// the gap between cache lookup and insertion. The data mutex remains short-
// lived, so source callbacks may recursively call/clear/inspect the cache.
static MEMOIZE_TURN: Mutex<()> = Mutex::new(());
thread_local! {
    static IN_MEMOIZE_TURN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
struct MemoizeTurn {
    guard: Option<std::sync::MutexGuard<'static, ()>>,
}
impl MemoizeTurn {
    fn enter() -> Self {
        if IN_MEMOIZE_TURN.with(std::cell::Cell::get) {
            return Self { guard: None };
        }
        let guard = MEMOIZE_TURN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        IN_MEMOIZE_TURN.with(|entered| entered.set(true));
        Self { guard: Some(guard) }
    }
}
impl Drop for MemoizeTurn {
    fn drop(&mut self) {
        if self.guard.is_some() {
            IN_MEMOIZE_TURN.with(|entered| entered.set(false));
        }
    }
}

/// Maps to: CC `utils/memoize.ts#LRUMemoizedFunction.cache`.
/// Mutex/IndexMap represent the shared LRU container. Public get is a peek;
/// only the memoized call promotes a hit.
pub struct LruMemoizedCache<Result>(Mutex<IndexMap<String, Result>>);

impl<Result: Clone> LruMemoizedCache<Result> {
    pub fn clear(&self) {
        let _turn = MemoizeTurn::enter();
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    pub fn size(&self) -> usize {
        let _turn = MemoizeTurn::enter();
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    pub fn delete(&self, key: &str) -> bool {
        let _turn = MemoizeTurn::enter();
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .shift_remove(key)
            .is_some()
    }

    pub fn get(&self, key: &str) -> Option<Result> {
        let _turn = MemoizeTurn::enter();
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(key)
            .cloned()
    }

    pub fn has(&self, key: &str) -> bool {
        let _turn = MemoizeTurn::enter();
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(key)
    }
}

/// Maps to: CC `utils/memoize.ts#LRUMemoizedFunction`.
/// Boxed callable fields carry the JS callable-with-cache object. Args is the
/// source argument tuple, and Clone results must retain any shared references.
pub struct LruMemoizedFunction<Args, Result> {
    function: Box<dyn Fn(&Args) -> Result + Send + Sync>,
    cache_fn: Box<dyn Fn(&Args) -> String + Send + Sync>,
    max_cache_size: usize,
    pub cache: LruMemoizedCache<Result>,
}

impl<Args, Result: Clone> LruMemoizedFunction<Args, Result> {
    /// Maps to: CC `utils/memoize.ts#memoizeWithLRU` inner `memoized` function.
    pub fn call(&self, args: &Args) -> Result {
        let _turn = MemoizeTurn::enter();
        let key = (self.cache_fn)(args);
        {
            let mut cache = self
                .cache
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(value) = cache.shift_remove(&key) {
                cache.insert(key, value.clone());
                return value;
            }
        }
        // A source callback may log or recursively call another memoized
        // operation; do not invoke it while holding a native non-reentrant lock.
        let value = (self.function)(args);
        let mut cache = self
            .cache
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.shift_remove(&key);
        cache.insert(key, value.clone());
        if cache.len() > self.max_cache_size {
            cache.shift_remove_index(0);
        }
        value
    }
}

/// Maps to: CC `utils/memoize.ts#memoizeWithLRU` (234–269).
/// None represents the optional maxCacheSize argument, whose default is 100.
pub fn memoize_with_lru<Args, Result: Clone>(
    function: impl Fn(&Args) -> Result + Send + Sync + 'static,
    cache_fn: impl Fn(&Args) -> String + Send + Sync + 'static,
    max_cache_size: Option<usize>,
) -> LruMemoizedFunction<Args, Result> {
    let max_cache_size = max_cache_size.unwrap_or(100);
    // lru-cache rejects zero max with no maxSize/ttl instead of disabling cache.
    assert!(
        max_cache_size > 0,
        "At least one of max, maxSize, or ttl is required"
    );
    LruMemoizedFunction {
        function: Box::new(function),
        cache_fn: Box::new(cache_fn),
        max_cache_size,
        cache: LruMemoizedCache(Mutex::new(IndexMap::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn memoize_with_lru_matches_official_key_cache_and_peek() {
        // CC utils/memoize.ts:246–266; cache.get is peek, calls promote.
        let count = Arc::new(AtomicUsize::new(0));
        let calls = count.clone();
        let memoized = memoize_with_lru(
            move |args: &(String, bool)| {
                calls.fetch_add(1, Ordering::SeqCst);
                Arc::new((args.0.clone(), args.1))
            },
            |args| args.0.clone(),
            Some(2),
        );
        let first = memoized.call(&("a".into(), false));
        let same = memoized.call(&("a".into(), true));
        assert!(Arc::ptr_eq(&first, &same));
        assert!(!same.1);
        memoized.call(&("b".into(), false));
        assert!(memoized.cache.get("a").is_some());
        memoized.call(&("c".into(), false));
        assert!(!memoized.cache.has("a"));
        assert_eq!(count.load(Ordering::SeqCst), 3);
        assert!(memoized.cache.delete("b"));
        assert_eq!(memoized.cache.size(), 1);
        memoized.cache.clear();
        assert_eq!(memoized.cache.size(), 0);
    }

    #[test]
    fn memoize_with_lru_matches_official_synchronous_turn_for_concurrent_callers() {
        // CC memoize.ts:246–255 cannot interleave another JS call between
        // lookup and cache insertion. Native caller threads must see one value.
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let memoized = Arc::new(memoize_with_lru(
            move |key: &u32| {
                count.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(40));
                Arc::new(*key)
            },
            |key| key.to_string(),
            Some(50),
        ));
        let ready = Arc::new(std::sync::Barrier::new(9));
        let (send, receive) = std::sync::mpsc::channel();
        let mut workers = Vec::new();
        for _ in 0..8 {
            let ready = ready.clone();
            let memoized = memoized.clone();
            let send = send.clone();
            workers.push(std::thread::spawn(move || {
                ready.wait();
                let _ = send.send(memoized.call(&1));
            }));
        }
        ready.wait();
        let values: Vec<_> = (0..8)
            .map(|_| {
                receive
                    .recv_timeout(std::time::Duration::from_secs(3))
                    .expect("synchronous memo turn completed")
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(values.iter().all(|value| Arc::ptr_eq(value, &values[0])));
    }

    #[test]
    fn memoize_with_lru_matches_official_reentrant_callback_and_cache_access() {
        // Source f executes outside lru-cache methods; nested calls and cache
        // operations are legal synchronously and must not deadlock in Rust.
        let (send, receive) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let memoized: Arc<LruMemoizedFunction<u32, Arc<u32>>> = Arc::new_cyclic(
                |weak: &std::sync::Weak<LruMemoizedFunction<u32, Arc<u32>>>| {
                    let weak = weak.clone();
                    memoize_with_lru(
                        move |key: &u32| {
                            if *key == 0 {
                                let owner = weak.upgrade().unwrap();
                                assert!(!owner.cache.has("0"));
                                owner.cache.clear();
                                Arc::new(*owner.call(&1) + 1)
                            } else {
                                Arc::new(*key)
                            }
                        },
                        |key| key.to_string(),
                        Some(2),
                    )
                },
            );
            let first = memoized.call(&0);
            let second = memoized.call(&0);
            send.send((*first, Arc::ptr_eq(&first, &second), memoized.cache.size()))
                .unwrap();
        });
        assert_eq!(
            receive
                .recv_timeout(std::time::Duration::from_secs(3))
                .expect("reentrant callback completed"),
            (2, true, 2)
        );
        worker.join().unwrap();
    }

    #[test]
    fn memoize_with_lru_matches_official_throw_not_cached_and_turn_released() {
        // CC memoize.ts:253–254: a thrown callback never reaches cache.set.
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let memoized = Arc::new(memoize_with_lru(
            move |key: &u32| {
                if count.fetch_add(1, Ordering::SeqCst) == 0 {
                    panic!("source callback throws");
                }
                Arc::new(*key)
            },
            |key| key.to_string(),
            Some(2),
        ));
        let first = memoized.clone();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| first.call(&1))).is_err());
        assert!(!memoized.cache.has("1"));
        let (send, receive) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            send.send(memoized.call(&1)).unwrap();
        });
        assert_eq!(
            *receive
                .recv_timeout(std::time::Duration::from_secs(3))
                .expect("panic must release the native turn"),
            1
        );
        worker.join().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
