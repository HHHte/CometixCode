//! Union-wide task predicates.
//!
//! Maps to: CC `tasks/types.ts`.

use crate::state::app_state_store::TaskState;

/// Maps to: CC `tasks/types.ts:37-46` `isBackgroundTask` — a task shows in the
/// background-tasks indicator iff:
/// 1. its status is `running` or `pending`, and
/// 2. it has not opted out via `isBackgrounded === false` (foreground tasks
///    are not yet "background tasks"; CC `:42` narrows with
///    `'isBackgrounded' in task`).
pub fn is_background_task(task: &TaskState) -> bool {
    let status = match task {
        TaskState::InProcessTeammate(task) => task.status.as_str(),
        TaskState::LocalShell(task) => task.status.as_str(),
        TaskState::Dream(task) => task.status.as_str(),
        TaskState::Other(task) => task.status.as_str(),
    };
    if status != "running" && status != "pending" {
        return false;
    }
    // CC `:42` — `'isBackgrounded' in task && task.isBackgrounded === false`.
    // The field always exists on LocalShell (plain bool), is optional on the
    // `Other` stub (`None` = field absent), and never exists on the teammate
    // or dream states.
    match task {
        TaskState::LocalShell(task) => task.is_backgrounded,
        TaskState::Other(task) => task.is_backgrounded != Some(false),
        TaskState::InProcessTeammate(_) | TaskState::Dream(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::app_state_store::TaskStateOther;

    fn other(status: &str, is_backgrounded: Option<bool>) -> TaskState {
        TaskState::Other(TaskStateOther {
            id: "t1".to_string(),
            task_type: "local_agent".to_string(),
            status: status.to_string(),
            description: String::new(),
            is_backgrounded,
            notified: false,
            retain: Some(false),
            evict_after: None,
            progress_tool_uses: None,
            progress_tokens: None,
        })
    }

    #[test]
    fn is_background_task_matches_cc_status_and_foreground_gates() {
        // CC types.ts:38-40 — running|pending only.
        assert!(is_background_task(&other("running", Some(true))));
        assert!(is_background_task(&other("pending", Some(true))));
        assert!(!is_background_task(&other("completed", Some(true))));
        assert!(!is_background_task(&other("failed", Some(true))));
        assert!(!is_background_task(&other("killed", Some(true))));
        // CC types.ts:42-44 — foreground excluded; absent field passes.
        assert!(!is_background_task(&other("running", Some(false))));
        assert!(is_background_task(&other("running", None)));
    }
}
