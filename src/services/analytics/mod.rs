//! Analytics service owners mirrored from CC `services/analytics/`.

pub mod config;
pub mod growthbook;

/// Maps to CC `services/analytics/index.ts#QueuedEvent` and eventQueue.
/// This port has no attached analytics backend. Preserve the source pre-sink
/// queue instead of sending telemetry to an invented endpoint.
static EVENT_QUEUE: std::sync::Mutex<Vec<(String, serde_json::Value)>> =
    std::sync::Mutex::new(Vec::new());

/// Maps to CC `services/analytics/index.ts#logEvent:138-151`, unattached-sink arm.
pub fn log_event(event_name: &str, mut metadata: serde_json::Value) {
    // JS undefined values are absent from the serialized metadata.
    if let Some(metadata) = metadata.as_object_mut() {
        metadata.retain(|_, value| !value.is_null());
    }
    EVENT_QUEUE
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .push((event_name.to_owned(), metadata));
}

/// Test-only observation of the existing unattached-sink queue; no draining or
/// replacement logger, so callers exercise the production log_event path.
#[cfg(test)]
pub(crate) fn queued_events_for_test() -> Vec<(String, serde_json::Value)> {
    EVENT_QUEUE
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
}
