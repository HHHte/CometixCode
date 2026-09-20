//! Process-lifetime async runtime handle for detached (`void`-style) work.
//!
//! Rust-only carrier — there is no CC file to map to. Classification per
//! PORTING.md § "Node-async → tokio" (necessity tiers): this module is
//! **derived (A3) from the A2 choice**, not a forced transformation. Node has
//! exactly ONE event loop per process, so CC's `void somePromise()`
//! (`AgentTool.tsx:999` `void runWithAgentContext(...)`, `resumeAgent.ts:230`
//! and friends) is detached work whose lifetime is the PROCESS, not the
//! generator that started it. A2 is the root decision that makes that implicit
//! loop need a name: `crate::query::spawn_query` gives every query actor a
//! PRIVATE `current_thread` runtime on its own OS thread and tears it down the
//! instant the actor future resolves, so inside tool execution
//! `tokio::runtime::Handle::try_current()` resolves to the *parent turn's*
//! runtime, and anything spawned onto it is destroyed when that turn ends —
//! the opposite of CC's `void` semantics. Had A2 taken its rejected
//! alternative (spawn actors onto the process runtime itself, requiring the
//! actor future to be `Send`), the ambient runtime would BE the process
//! runtime everywhere and this file would not need to exist — it names Node's
//! implicit "only loop" explicitly, it is not a new mechanism.
//!
//! Per A3, **every entrypoint publishes**: interactive `main.rs` right after
//! building the render-loop runtime, and headless `cli/print.rs` right after
//! building its session runtime (the print path branches at `cli/dispatch.rs`
//! before `main.rs::run()` ever executes, so it must publish its own). Detached
//! work that must outlive the spawning turn asks for
//! [`runtime_handle_for_detached_work`].

use std::sync::OnceLock;
use tokio::runtime::{Handle, RuntimeFlavor};

static PROCESS_RUNTIME: OnceLock<Handle> = OnceLock::new();

/// Publish the process-lifetime runtime. Called once per entrypoint
/// (`main.rs`, `cli/print.rs`) right after the runtime is built; later calls
/// are ignored so a test harness or a second entrypoint cannot repoint live
/// detached work at a dying runtime.
pub fn set_process_runtime_handle(handle: Handle) {
    let _ = PROCESS_RUNTIME.set(handle);
}

/// The published process runtime, if an entrypoint has published one.
pub fn process_runtime_handle() -> Option<Handle> {
    PROCESS_RUNTIME.get().cloned()
}

/// Handle to spawn detached work that must outlive the current turn. (The
/// WORK is detached, not the handle.)
///
/// Prefers the published process runtime (CC's single event loop). Falls back
/// to the ambient runtime for entrypoints that never publish — unit tests and
/// embedded/SDK callers — but only when that ambient runtime is
/// multi-threaded: every per-query runtime this port builds is
/// `current_thread` (`query.rs#spawn_query`), so an UNPUBLISHED
/// `current_thread` ambient is almost certainly a turn-scoped runtime, and
/// handing it out re-creates #150 (detached work silently destroyed with the
/// turn). Refusing (`None`) makes those callers fail loudly instead. The
/// published handle is checked first and is exempt from the flavor guard:
/// print's session runtime is `current_thread` yet process-lifetime
/// (block_on-driven for the whole session), and it arrives here via the
/// `OnceLock`, not the fallback.
pub fn runtime_handle_for_detached_work() -> Option<Handle> {
    process_runtime_handle().or_else(|| {
        Handle::try_current()
            .ok()
            .filter(|handle| handle.runtime_flavor() != RuntimeFlavor::CurrentThread)
    })
}

/// Run an async future to completion from SYNC code that may already be inside
/// a runtime. The one safe shape for this port's sync bridges.
///
/// Classification per PORTING.md § "Node-async → tokio" A4: **derived**
/// from A2 plus the choice to keep some interfaces sync (CC has no
/// sync-waits-on-async at all — JS is async throughout). If either root
/// decision changed — actors on the process runtime, or the sync interfaces
/// going async — these branches would collapse.
///
/// `tokio::task::block_in_place` PANICS on a `current_thread` runtime
/// ("can call blocking only when running on the multi-threaded runtime"), and
/// every query actor is exactly that ([`crate::query::spawn_query`] builds
/// `new_current_thread`). So a bridge written as
/// `block_in_place(|| handle.block_on(fut))` is a live panic for any sync
/// permission/classifier path a subagent reaches — the caller just has to be
/// unlucky enough to need it.
///
/// The three cases, in the order they are checked:
/// 1. Ambient MULTI-thread runtime — `block_in_place` is legal and cheapest:
///    it moves the current worker out of the pool so the runtime keeps
///    progressing while this thread blocks.
/// 2. Ambient CURRENT-thread runtime — blocking the only thread would deadlock
///    and `block_in_place` would panic, so the future is handed to a scratch
///    thread with its own runtime and this thread waits on the join.
/// 3. No runtime at all — plain `Runtime::new().block_on`.
///
/// `None` means no runtime could be created; callers own the fallback, because
/// the right answer differs (deny for a permission gate, fail-closed for a
/// classifier).
pub fn block_on_from_sync<F>(future: F) -> Option<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    match Handle::try_current() {
        Ok(handle) => match handle.runtime_flavor() {
            RuntimeFlavor::CurrentThread => {
                // Case 2: a scratch runtime on its own thread. The ambient
                // current_thread runtime keeps its single thread free to be
                // driven by whoever owns it.
                std::thread::spawn(move || {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .ok()
                        .map(|runtime| runtime.block_on(future))
                })
                .join()
                .ok()
                .flatten()
            }
            // Case 1.
            _ => Some(tokio::task::block_in_place(|| handle.block_on(future))),
        },
        // Case 3.
        Err(_) => tokio::runtime::Runtime::new()
            .ok()
            .map(|runtime| runtime.block_on(future)),
    }
}

/// Explicit test-entrypoint fixture for components/services that schedule
/// process-lifetime work, such as notification timers. The runtime itself must
/// outlive each component's render loop; publishing a test-local handle alone
/// would leave PROCESS_RUNTIME pointing at a dropped executor.
///
/// Tests run in separate processes under nextest. Tests of unpublished or
/// print/query runtime topologies deliberately do not call this fixture.
#[cfg(test)]
pub(crate) fn initialize_test_process_runtime() {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    let runtime = RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("test process runtime")
    });
    set_process_runtime_handle(runtime.handle().clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// The regression this module exists for: a detached future spawned from
    /// inside a short-lived per-query runtime must survive that runtime being
    /// dropped, exactly as a `void` promise survives a CC generator returning.
    ///
    /// Without [`runtime_handle_for_detached_work`] the spawn lands on the
    /// inner `current_thread` runtime and is destroyed with it — which is
    /// precisely how a background Agent stopped ever reaching a terminal state.
    #[test]
    fn detached_work_outlives_the_spawning_query_runtime() {
        let process_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("process runtime");
        set_process_runtime_handle(process_runtime.handle().clone());

        let finished = Arc::new(AtomicBool::new(false));
        let finished_for_task = Arc::clone(&finished);
        let (started_tx, started_rx) = std::sync::mpsc::channel::<()>();
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();

        // Stands in for `spawn_query`'s private per-query runtime: built here,
        // dropped as soon as its `block_on` returns.
        {
            let query_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("per-query runtime");
            query_runtime.block_on(async {
                let handle = runtime_handle_for_detached_work().expect("detached handle");
                handle.spawn(async move {
                    let _ = started_tx.send(());
                    // Only completes after the spawning runtime is gone.
                    let _ = tokio::task::spawn_blocking(move || release_rx.recv()).await;
                    finished_for_task.store(true, Ordering::SeqCst);
                });
            });
            // `query_runtime` drops here — the parent turn is over.
        }

        started_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("detached task must be polled at least once");
        let _ = release_tx.send(());

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !finished.load(Ordering::SeqCst) && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            finished.load(Ordering::SeqCst),
            "detached work must keep running after the per-query runtime is dropped"
        );
    }

    /// The headless (#150 print-mode) topology: `cli/print.rs` builds ONE
    /// `current_thread` session runtime, publishes its handle, and drives it
    /// with `block_on` for the whole session, while `query_engine` tool
    /// execution runs inside a NESTED per-query `current_thread` runtime.
    /// Detached work resolved from inside that nested runtime must land on the
    /// published session runtime and survive the nested runtime's drop.
    ///
    /// Isolation note: the `OnceLock` is process-global and
    /// [`detached_work_outlives_the_spawning_query_runtime`] publishes a
    /// multi_thread handle. Under nextest (the gate) every test is its own
    /// process, so the publish below always wins; under plain `cargo test`
    /// another test may have won the race, in which case the print topology
    /// cannot be reproduced and the test logs a note and skips — detected by
    /// flavor, since this is the only test in this crate that publishes a
    /// `current_thread` handle.
    #[test]
    fn published_print_session_runtime_carries_detached_work_from_a_nested_query_runtime() {
        // What print.rs does at startup.
        let session_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("print-style session runtime");
        if process_runtime_handle().is_none() {
            set_process_runtime_handle(session_runtime.handle().clone());
        }
        let published = process_runtime_handle().expect("published handle");
        if published.runtime_flavor() != RuntimeFlavor::CurrentThread {
            eprintln!(
                "note: another test in this process already published a multi_thread \
                 handle; the print topology is not reproducible here. Skipping — the \
                 gate (nextest) runs this test in its own process where the publish \
                 always wins."
            );
            return;
        }

        let (done_tx, done_rx) = tokio::sync::oneshot::channel::<()>();
        // Stands in for the per-query runtime `spawn_query` builds under
        // `query_engine::ask` — tool execution's ambient runtime in print mode.
        {
            let query_runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("per-query runtime");
            query_runtime.block_on(async {
                let handle = runtime_handle_for_detached_work()
                    .expect("must resolve to the published session handle");
                // If this handle were the ambient per-query runtime (the #150
                // bug), the task would be destroyed with it below and `done_rx`
                // would resolve to RecvError instead of ().
                handle.spawn(async move {
                    let _ = done_tx.send(());
                });
            });
            // Per-query runtime drops here; the detached task must not.
        }

        // Drive the published session runtime the way print.rs does — via
        // block_on. A current_thread runtime only makes progress while driven,
        // so this is also the proof the spawn landed on THIS runtime.
        let outcome = session_runtime.block_on(async {
            tokio::time::timeout(std::time::Duration::from_secs(5), done_rx).await
        });
        assert!(
            outcome
                .expect("detached task must be driven by the published session runtime")
                .is_ok(),
            "detached work must survive the nested per-query runtime's drop"
        );
    }

    /// The fallback refusal: with NO published handle, asking for a detached
    /// handle from inside a bare `current_thread` runtime must return `None` —
    /// an unpublished `current_thread` ambient is almost certainly a
    /// turn-scoped per-query runtime, and handing it out re-creates #150.
    ///
    /// Isolation note: the refusal is only observable while the process-global
    /// `OnceLock` is empty. Under nextest (the gate) this test runs in its own
    /// process and nothing here publishes, so the refusal branch always runs;
    /// under plain `cargo test` a sibling test may have published first, in
    /// which case the refusal is unreachable and the test logs a note and
    /// falls back to the invariant that holds in EVERY `OnceLock` state: a
    /// returned handle is never an unpublished current_thread ambient.
    #[test]
    fn runtime_handle_for_detached_work_refuses_an_unpublished_current_thread_ambient() {
        let bare_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("bare current_thread runtime");
        bare_runtime.block_on(async {
            let already_published = process_runtime_handle().is_some();
            let handle = runtime_handle_for_detached_work();

            // Always-valid invariant, independent of OnceLock state.
            if let Some(handle) = &handle {
                assert!(
                    already_published || handle.runtime_flavor() != RuntimeFlavor::CurrentThread,
                    "an unpublished current_thread ambient must never be handed out"
                );
            }

            if already_published {
                eprintln!(
                    "note: a sibling test already published the process handle in this \
                     process; the refusal branch is unreachable here. Skipping — the \
                     gate (nextest) runs this test in its own unpublished process."
                );
                return;
            }
            assert!(
                handle.is_none(),
                "with no published process runtime, a current_thread ambient must be \
                 refused, not handed out as a detach target"
            );
        });
    }

    /// `tokio::task::block_in_place` PANICS on a `current_thread` runtime, and
    /// EVERY query actor is one (`query.rs#spawn_query` builds
    /// `new_current_thread`). The port's sync bridges — the headless-ask tail
    /// and the auto-mode classifier — are reached from inside tool execution,
    /// i.e. from inside that actor, so a hand-rolled
    /// `block_in_place(|| handle.block_on(…))` there is a live crash waiting
    /// for the first caller that needs it.
    ///
    /// This pins all three shapes [`block_on_from_sync`] must survive.
    #[test]
    fn block_on_from_sync_survives_every_runtime_shape() {
        // 1. Inside a current_thread runtime — the query-actor case, the one
        //    that used to panic.
        let actor_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("actor runtime");
        let inside_current_thread =
            actor_runtime.block_on(async { block_on_from_sync(async { 41 + 1 }) });
        assert_eq!(inside_current_thread, Some(42));

        // 2. Inside a multi_thread runtime — block_in_place is legal there.
        let pool_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("pool runtime");
        let inside_multi_thread =
            pool_runtime.block_on(async { block_on_from_sync(async { "ok" }) });
        assert_eq!(inside_multi_thread, Some("ok"));

        // 3. No ambient runtime at all.
        assert_eq!(block_on_from_sync(async { 7u8 }), Some(7));
    }
}
