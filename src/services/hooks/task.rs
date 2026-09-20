//! Task lifecycle hook execution.
//! Maps to: CC utils/hooks.ts:3745-3817 (executeTaskCreatedHooks, executeTaskCompletedHooks).

use super::exec::exec_command_hook;
use super::matching::get_matching_hooks;
use super::parsing::{ParsedHookOutput, parse_hook_output, process_hook_json_output};
use super::{
    HookBlockingError, HookContext, HookEvent, HookInputBuilder, HookOutcome, HookResult,
    RegisteredHooks,
};
use std::time::Duration;

const TOOL_HOOK_TIMEOUT_MS: u64 = 30_000;

/// The `...createBaseHookInput(permissionMode)` spread both task builders open
/// with (CC `hooks.ts:3757`, `:3801`).
///
/// Both CC call sites pass `undefined` for `permissionMode` — `TaskCreateTool.ts`
/// `executeTaskCreatedHooks(taskId, subject, description, getAgentName(),
/// getTeamName(), undefined, …)` and the identical `TaskUpdateTool.ts:235-245`
/// — and neither passes any agent info, so `permission_mode`, `agent_id` and
/// `agent_type` are all absent on the wire.
fn task_base_input() -> HookInputBuilder {
    HookInputBuilder::from_base(super::create_base_hook_input_object(&HookContext::default()))
}

/// Maps to: CC `utils/hooks.ts:1914-1918` `getTaskCreatedHookMessage(...)`.
pub fn get_task_created_hook_message(blocking_error: &HookBlockingError) -> String {
    format!(
        "TaskCreated hook feedback:\n{}",
        blocking_error.blocking_error
    )
}

/// Maps to: CC `utils/hooks.ts:1925-1929` `getTaskCompletedHookMessage(...)`.
pub fn get_task_completed_hook_message(blocking_error: &HookBlockingError) -> String {
    format!(
        "TaskCompleted hook feedback:\n{}",
        blocking_error.blocking_error
    )
}

/// Maps to: CC `executeTaskCreatedHooks()` (hooks.ts:3745-3773).
pub async fn execute_task_created_hooks(
    config: &RegisteredHooks,
    task_id: &str,
    task_subject: &str,
    task_description: Option<&str>,
    teammate_name: Option<&str>,
    team_name: Option<&str>,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    let input = task_base_input()
        .set("hook_event_name", "TaskCreated")
        .set("task_id", task_id)
        .set("task_subject", task_subject)
        .set_optional("task_description", task_description)
        .set_optional("teammate_name", teammate_name)
        .set_optional("team_name", team_name)
        .build();
    execute_task_event(config, HookEvent::TaskCreated, &input, base_env).await
}

/// Maps to: CC `executeTaskCompletedHooks()` (hooks.ts:3789-3817).
pub async fn execute_task_completed_hooks(
    config: &RegisteredHooks,
    task_id: &str,
    task_subject: &str,
    task_description: Option<&str>,
    teammate_name: Option<&str>,
    team_name: Option<&str>,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    let input = task_base_input()
        .set("hook_event_name", "TaskCompleted")
        .set("task_id", task_id)
        .set("task_subject", task_subject)
        .set_optional("task_description", task_description)
        .set_optional("teammate_name", teammate_name)
        .set_optional("team_name", team_name)
        .build();
    execute_task_event(config, HookEvent::TaskCompleted, &input, base_env).await
}

async fn execute_task_event(
    config: &RegisteredHooks,
    event: HookEvent,
    input: &serde_json::Value,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    // Maps to: CC `hooks.ts:1978-1999`. Neither `executeTaskCreatedHooks`
    // (`:3766-3772`) nor `executeTaskCompletedHooks` (`:3810-3816`) passes a
    // `matchQuery`, so CC's `hookName` is the bare event name.
    if crate::services::hooks::should_skip_hook_execution(event, "") {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, event, "", None);
    if matched.is_empty() {
        return Vec::new();
    }

    let input_str = input.to_string();
    let mut results = Vec::new();
    for hook in &matched {
        // CC hooks.ts:2147 — callback hooks resolve via the SDK consumer.
        let result = match &hook.hook {
            crate::schemas::hooks::RegisteredHook::Callback(callback) => {
                let json = super::exec::exec_callback_hook(callback, &input_str, None).await;
                process_hook_json_output(&json, "callback")
            }
            crate::schemas::hooks::RegisteredHook::Command(command) => {
                let timeout = command.timeout.unwrap_or(TOOL_HOOK_TIMEOUT_MS / 1000) * 1000;
                let exec_result = exec_command_hook(
                    &command.command,
                    &input_str,
                    Duration::from_millis(timeout),
                    base_env.clone(),
                    hook.plugin_root.as_deref(),
                    hook.plugin_id.as_deref(),
                    None,
                )
                .await;

                match parse_hook_output(&exec_result.stdout) {
                    ParsedHookOutput::Json(json) => {
                        process_hook_json_output(&json, &command.command)
                    }
                    ParsedHookOutput::PlainText(text) => HookResult {
                        system_message: Some(text),
                        ..Default::default()
                    },
                    _ => HookResult::default(),
                }
            }
        };

        let should_stop = result.prevent_continuation || result.outcome == HookOutcome::Blocking;
        results.push(result);
        if should_stop {
            break;
        }
    }
    results
}
