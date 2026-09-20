//! SDK `task_progress` event emission.
//!
//! Maps to: CC `utils/task/sdkProgress.ts`.
//!
//! `utils/sdkEventQueue.ts` is not ported yet, so events are retained in a
//! process-local buffer for tests/callers instead of being pushed to an SDK
//! transport.

use std::sync::{Mutex, OnceLock};

/// Maps to: CC `sdkProgress.ts#emitTaskProgress` params.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskProgressEvent {
    pub task_id: String,
    pub tool_use_id: Option<String>,
    pub description: String,
    pub start_time_ms: u64,
    pub total_tokens: u64,
    pub tool_uses: usize,
    pub last_tool_name: Option<String>,
    pub summary: Option<String>,
    pub duration_ms: u64,
}

fn progress_buffer() -> &'static Mutex<Vec<TaskProgressEvent>> {
    static BUFFER: OnceLock<Mutex<Vec<TaskProgressEvent>>> = OnceLock::new();
    BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

/// Maps to: CC `utils/task/sdkProgress.ts#emitTaskProgress`.
pub fn emit_task_progress(params: EmitTaskProgressParams<'_>) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let event = TaskProgressEvent {
        task_id: params.task_id.to_string(),
        tool_use_id: params.tool_use_id.map(ToOwned::to_owned),
        description: params.description.to_string(),
        start_time_ms: params.start_time_ms,
        total_tokens: params.total_tokens,
        tool_uses: params.tool_uses,
        last_tool_name: params.last_tool_name.map(ToOwned::to_owned),
        summary: params.summary.map(ToOwned::to_owned),
        duration_ms: now_ms.saturating_sub(params.start_time_ms),
    };
    if let Ok(mut buffer) = progress_buffer().lock() {
        buffer.push(event);
    }
}

pub struct EmitTaskProgressParams<'a> {
    pub task_id: &'a str,
    pub tool_use_id: Option<&'a str>,
    pub description: &'a str,
    pub start_time_ms: u64,
    pub total_tokens: u64,
    pub tool_uses: usize,
    pub last_tool_name: Option<&'a str>,
    pub summary: Option<&'a str>,
}

/// Test/helper seam until `sdkEventQueue` is ported.
#[cfg(test)]
pub fn take_task_progress_events_for_test() -> Vec<TaskProgressEvent> {
    progress_buffer()
        .lock()
        .map(|mut buffer| std::mem::take(&mut *buffer))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_task_progress_records_event_shape() {
        let _ = take_task_progress_events_for_test();
        emit_task_progress(EmitTaskProgressParams {
            task_id: "agent-1",
            tool_use_id: Some("toolu_1"),
            description: "inspect",
            start_time_ms: 1,
            total_tokens: 12,
            tool_uses: 3,
            last_tool_name: Some("Read"),
            summary: None,
        });
        let events = take_task_progress_events_for_test();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].task_id, "agent-1");
        assert_eq!(events[0].last_tool_name.as_deref(), Some("Read"));
        assert_eq!(events[0].tool_uses, 3);
    }
}
