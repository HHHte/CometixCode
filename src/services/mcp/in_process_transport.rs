//! In-process MCP transport pair boundary.
//! Maps to: CC `services/mcp/InProcessTransport.ts`.
//!
//! Upstream purpose:
//! - `InProcessTransport` is one side of a linked client/server MCP transport.
//! - `createLinkedTransportPair()` returns `[clientTransport, serverTransport]`.
//! - `send()` on one side asynchronously calls `onmessage` on the peer.
//! - `close()` on either side closes both sides.

use serde_json::Value;
use std::sync::{Arc, Mutex, Weak};

/// Maps to: CC `Transport.onmessage` callback.
pub type InProcessMessageHandler = Arc<dyn Fn(Value) + Send + Sync + 'static>;
/// Maps to: CC `Transport.onclose` callback.
pub type InProcessCloseHandler = Arc<dyn Fn() + Send + Sync + 'static>;
/// Maps to: CC `Transport.onerror` callback.
pub type InProcessErrorHandler = Arc<dyn Fn(anyhow::Error) + Send + Sync + 'static>;

#[derive(Default)]
struct InProcessTransportInner {
    peer: Option<Weak<Mutex<InProcessTransportInner>>>,
    closed: bool,
    on_message: Option<InProcessMessageHandler>,
    on_close: Option<InProcessCloseHandler>,
    #[allow(dead_code)]
    on_error: Option<InProcessErrorHandler>,
}

/// Official in-process linked transport.
/// Maps to: CC `services/mcp/InProcessTransport.ts#InProcessTransport`.
#[derive(Clone, Default)]
pub struct InProcessTransport {
    inner: Arc<Mutex<InProcessTransportInner>>,
}

impl InProcessTransport {
    fn set_peer(&self, peer: &InProcessTransport) {
        self.inner.lock().expect("transport lock poisoned").peer =
            Some(Arc::downgrade(&peer.inner));
    }

    /// Maps to: CC `InProcessTransport.start()` (empty start method).
    pub async fn start(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Maps to: CC assignable `transport.onmessage` field.
    pub fn set_on_message(&self, handler: Option<InProcessMessageHandler>) {
        self.inner
            .lock()
            .expect("transport lock poisoned")
            .on_message = handler;
    }

    /// Maps to: CC assignable `transport.onclose` field.
    pub fn set_on_close(&self, handler: Option<InProcessCloseHandler>) {
        self.inner.lock().expect("transport lock poisoned").on_close = handler;
    }

    /// Maps to: CC assignable `transport.onerror` field.
    pub fn set_on_error(&self, handler: Option<InProcessErrorHandler>) {
        self.inner.lock().expect("transport lock poisoned").on_error = handler;
    }

    /// Maps to: CC `InProcessTransport.send(message)`: deliver to the peer's
    /// `onmessage` asynchronously, preserving request/response re-entrancy
    /// behavior instead of returning a deferred stub.
    pub async fn send(&self, message: Value) -> anyhow::Result<()> {
        let peer_handler = {
            let inner = self.inner.lock().expect("transport lock poisoned");
            if inner.closed {
                anyhow::bail!("Transport is closed");
            }
            inner
                .peer
                .as_ref()
                .and_then(Weak::upgrade)
                .and_then(|peer| {
                    peer.lock()
                        .expect("transport lock poisoned")
                        .on_message
                        .clone()
                })
        };

        if let Some(handler) = peer_handler {
            // Maps to CC `InProcessTransport.ts:30-34`: delivery is
            // `queueMicrotask(() => this.peer?.onmessage?.(message))` — async
            // but GUARANTEED on the process loop. A3 audit (PORTING.md §
            // "Node-async → tokio"): the old `try_current()` spawned onto whatever runtime
            // `send()` ran in — inside a tool call that is the turn's private
            // runtime, and a message in flight when the turn resolves was
            // silently dropped (a lost JSONRPC response = a hung request).
            // Known deviation: queueMicrotask is FIFO; a multi-thread process
            // runtime does not order concurrent spawns. JSONRPC correlation is
            // by id, so reordering is tolerable; do not rely on cross-message
            // order here.
            if let Some(handle) = crate::utils::process_runtime::runtime_handle_for_detached_work()
            {
                handle.spawn(async move {
                    (handler)(message);
                });
            } else {
                std::thread::spawn(move || {
                    (handler)(message);
                });
            }
        }
        Ok(())
    }

    /// Maps to: CC `InProcessTransport.close()`: closing either side closes
    /// both linked transports and fires each side's `onclose` once.
    pub async fn close(&self) -> anyhow::Result<()> {
        let (own_close, peer) = {
            let mut inner = self.inner.lock().expect("transport lock poisoned");
            if inner.closed {
                return Ok(());
            }
            inner.closed = true;
            (
                inner.on_close.clone(),
                inner.peer.as_ref().and_then(Weak::upgrade),
            )
        };

        if let Some(on_close) = own_close {
            on_close();
        }

        if let Some(peer) = peer {
            let peer_close = {
                let mut peer_inner = peer.lock().expect("transport lock poisoned");
                if peer_inner.closed {
                    None
                } else {
                    peer_inner.closed = true;
                    peer_inner.on_close.clone()
                }
            };
            if let Some(on_close) = peer_close {
                on_close();
            }
        }
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.inner.lock().expect("transport lock poisoned").closed
    }
}

/// Official linked-pair factory.
/// Maps to: CC `services/mcp/InProcessTransport.ts#createLinkedTransportPair`.
pub fn create_linked_transport_pair() -> (InProcessTransport, InProcessTransport) {
    let client = InProcessTransport::default();
    let server = InProcessTransport::default();
    client.set_peer(&server);
    server.set_peer(&client);
    (client, server)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn in_process_transport_delivers_to_peer_onmessage_asynchronously() {
        let (client, server) = create_linked_transport_pair();
        let (tx, rx) = async_channel::bounded(1);
        server.set_on_message(Some(Arc::new(move |message| {
            tx.try_send(message).unwrap();
        })));

        client
            .send(serde_json::json!({ "jsonrpc": "2.0", "method": "ping" }))
            .await
            .unwrap();

        let message = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(message["method"].as_str(), Some("ping"));
        assert!(!client.is_closed());
        assert!(!server.is_closed());
    }

    #[tokio::test]
    async fn in_process_transport_close_closes_both_sides_once() {
        let (client, server) = create_linked_transport_pair();
        let closed = Arc::new(AtomicUsize::new(0));
        let client_closed = closed.clone();
        client.set_on_close(Some(Arc::new(move || {
            client_closed.fetch_add(1, Ordering::SeqCst);
        })));
        let server_closed = closed.clone();
        server.set_on_close(Some(Arc::new(move || {
            server_closed.fetch_add(1, Ordering::SeqCst);
        })));

        client.close().await.unwrap();
        client.close().await.unwrap();

        assert!(client.is_closed());
        assert!(server.is_closed());
        assert_eq!(closed.load(Ordering::SeqCst), 2);
        let error = server
            .send(serde_json::json!({ "jsonrpc": "2.0", "method": "after-close" }))
            .await
            .unwrap_err();
        assert_eq!(error.to_string(), "Transport is closed");
    }
}
