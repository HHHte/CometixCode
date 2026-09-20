//! Pure LocalShellTask kill helpers.
//!
//! Maps to: CC `tasks/LocalShellTask/killShellTasks.ts:1-77`.

use crate::state::app_state_store::TaskState;
use crate::state::store::{AppStore, UpdateDecision};

/// Maps to CC `killTask(taskId, setAppState)`.
pub fn kill_task(task_id: &str, preferred: Option<&AppStore>) -> bool {
    let Some(store) = super::task_store(task_id, preferred) else {
        return false;
    };
    // P3 §3a: a missing or non-running task returns `prev` untouched in CC;
    // Same carries the (None, false) result (former out-params).
    let (shell_command, killed) = store.set_state(|prev| {
        match prev.tasks.get(task_id).map(|task| task.as_ref()) {
            Some(TaskState::LocalShell(task)) if task.status == "running" => {}
            _ => return UpdateDecision::Same((None, false)),
        }
        let mut next = (**prev).clone();
        // P4 identity: make_mut(map) + make_mut(task Arc) = CC's double
        // spread (fresh map + fresh per-task reference).
        let shell_command = match std::sync::Arc::make_mut(&mut next.tasks).get_mut(task_id) {
            Some(entry) => match std::sync::Arc::make_mut(entry) {
                TaskState::LocalShell(task) => {
                    task.status = "killed".to_string();
                    task.notified = true;
                    task.end_time_ms = Some(chrono::Utc::now().timestamp_millis());
                    task.shell_command.take()
                }
                _ => None,
            },
            None => None,
        };
        UpdateDecision::Replace {
            next: std::sync::Arc::new(next),
            result: (shell_command, true),
        }
    });
    if let Some(shell_command) = shell_command {
        shell_command.kill();
        shell_command.cleanup();
    }
    killed
}

/// Process-shutdown equivalent of each LocalShellTask cleanup-registry entry.
pub fn kill_all_shell_tasks() -> usize {
    let ids = super::TASK_STORES
        .lock()
        .map(|stores| stores.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    ids.into_iter().filter(|id| kill_task(id, None)).count()
}

/// Maps to CC `killShellTasksForAgent(agentId, getAppState, setAppState)`.
pub fn kill_shell_tasks_for_agent(agent_id: &str) -> usize {
    let ids = super::TASK_STORES
        .lock()
        .map(|stores| stores.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut killed = 0;
    for id in ids {
        let Some(task) = super::task_state(&id, None) else {
            continue;
        };
        if task.agent_id.as_deref() == Some(agent_id)
            && task.status == "running"
            && kill_task(&id, None)
        {
            killed += 1;
        }
    }
    crate::utils::message_queue_manager::dequeue_all_matching(|command| {
        command.agent_id.as_deref() == Some(agent_id)
    });
    killed
}
