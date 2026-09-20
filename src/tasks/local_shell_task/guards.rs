//! Local shell task state and guard.
//!
//! Maps to: CC `tasks/LocalShellTask/guards.ts:1-42`.

use crate::utils::shell_command::ShellCommand;

pub type BashTaskKind = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalShellTaskResult {
    pub code: i32,
    pub interrupted: bool,
}

/// Maps to CC `LocalShellTaskState`.
#[derive(Clone, Debug)]
pub struct LocalShellTaskState {
    pub id: String,
    pub task_type: String,
    pub status: String,
    pub description: String,
    pub command: String,
    pub result: Option<LocalShellTaskResult>,
    pub notified: bool,
    pub shell_command: Option<ShellCommand>,
    pub last_reported_total_lines: usize,
    pub is_backgrounded: bool,
    pub agent_id: Option<String>,
    pub tool_use_id: Option<String>,
    pub kind: Option<BashTaskKind>,
    pub start_time_ms: i64,
    pub end_time_ms: Option<i64>,
}

impl PartialEq for LocalShellTaskState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.task_type == other.task_type
            && self.status == other.status
            && self.description == other.description
            && self.command == other.command
            && self.result == other.result
            && self.notified == other.notified
            && self.shell_command == other.shell_command
            && self.last_reported_total_lines == other.last_reported_total_lines
            && self.is_backgrounded == other.is_backgrounded
            && self.agent_id == other.agent_id
            && self.tool_use_id == other.tool_use_id
            && self.kind == other.kind
            && self.start_time_ms == other.start_time_ms
            && self.end_time_ms == other.end_time_ms
    }
}

impl Eq for LocalShellTaskState {}

/// Maps to CC `isLocalShellTask(task)`; Rust's enum discriminant supplies the
/// runtime guard, so this helper preserves the source name for consumers.
pub fn is_local_shell_task(
    task: &crate::state::app_state_store::TaskState,
) -> Option<&LocalShellTaskState> {
    match task {
        crate::state::app_state_store::TaskState::LocalShell(task) => Some(task),
        _ => None,
    }
}
