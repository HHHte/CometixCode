//! Global graceful-shutdown cleanup registry.
//!
//! Maps to: CC `utils/cleanupRegistry.ts`.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

pub type CleanupFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
type CleanupFunction = Arc<dyn Fn() -> CleanupFuture + Send + Sync + 'static>;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static CLEANUP_FUNCTIONS: LazyLock<Mutex<HashMap<u64, CleanupFunction>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Handle corresponding to CC's unregister closure.
#[derive(Debug)]
pub struct CleanupRegistration {
    id: u64,
}

impl CleanupRegistration {
    pub fn unregister(&self) -> bool {
        CLEANUP_FUNCTIONS
            .lock()
            .map(|mut functions| functions.remove(&self.id).is_some())
            .unwrap_or(false)
    }
}

/// Maps to CC `registerCleanup(cleanupFn)`.
pub fn register_cleanup<F, Fut>(cleanup: F) -> CleanupRegistration
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let function: CleanupFunction = Arc::new(move || Box::pin(cleanup()));
    if let Ok(mut functions) = CLEANUP_FUNCTIONS.lock() {
        functions.insert(id, function);
    }
    CleanupRegistration { id }
}

fn snapshot() -> Vec<CleanupFunction> {
    CLEANUP_FUNCTIONS
        .lock()
        .map(|functions| functions.values().cloned().collect())
        .unwrap_or_default()
}

/// Maps to CC `runCleanupFunctions()` and runs one snapshot concurrently.
pub async fn run_cleanup_functions() {
    futures::future::join_all(snapshot().into_iter().map(|function| function())).await;
}

/// Synchronous process-exit bridge for Rust's `std::process::exit`, which does
/// not run destructors. Bash/task cleanup callbacks are synchronous futures,
/// so this preserves them even on top-level error exits.
pub fn run_cleanup_functions_sync() {
    futures::executor::block_on(run_cleanup_functions());
}

pub fn exit_process(code: i32) -> ! {
    run_cleanup_functions_sync();
    std::process::exit(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_runs_and_can_be_unregistered() {
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = Arc::clone(&count);
        let registration = register_cleanup(move || {
            let observed = Arc::clone(&observed);
            async move {
                observed.fetch_add(1, Ordering::SeqCst);
            }
        });
        let function = CLEANUP_FUNCTIONS
            .lock()
            .unwrap()
            .remove(&registration.id)
            .expect("registration is stored");
        futures::executor::block_on(function());
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(!registration.unregister());
    }
}
