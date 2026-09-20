//! MCP monitor task state (MONITOR_TOOL).
//!
//! Maps to: CC `tasks/MonitorMcpTask/MonitorMcpTask.ts:1-4`.
//!
//! NO-SOURCE SEAM: the CC 2.1.88 rebuild carries only a 4-line
//! `@generated-stub` for this file (missing from the sourcemap; the type below
//! is that stub verbatim). The real MonitorMcpTask runtime
//! (`killMonitorMcp`, the MONITOR_TOOL feature gate wiring) has no portable
//! source; do not invent it here.

/// Maps to: CC generated-stub `MonitorMcpTaskState` —
/// `{ status: 'pending' | 'running' | 'completed' | 'failed'; serverName?: string }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorMcpTaskState {
    pub status: String,
    pub server_name: Option<String>,
}
