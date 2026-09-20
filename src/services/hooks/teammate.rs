//! Teammate/Subagent hook execution.
//! Maps to: CC utils/hooks.ts:3709-3952 (executeTeammateIdleHooks, executeSubagentStartHooks)
//! and utils/hooks.ts:3653-3697 (`executeStopHooks` SubagentStop branch).

use super::exec::exec_command_hook;
use super::matching::get_matching_hooks;
use super::parsing::{ParsedHookOutput, parse_hook_output, process_hook_json_output};
use super::{
    CommandExecResult, HookContext, HookEvent, HookInputBuilder, HookOutcome, HookResult,
    RegisteredHooks,
};
use std::time::{Duration, Instant};

const TOOL_HOOK_TIMEOUT_MS: u64 = 30_000;

/// The `...createBaseHookInput(...)` spread the teammate/subagent builders open
/// with. None of the three CC sites passes agent info (`hooks.ts:3717`,
/// `:3672`, `:3939`), so `agent_id`/`agent_type` reach the wire only through
/// each event's own required keys.
fn teammate_base_input(permission_mode: Option<&str>) -> HookInputBuilder {
    HookInputBuilder::from_base(super::create_base_hook_input_object(&HookContext {
        permission_mode: permission_mode.map(str::to_string),
        ..Default::default()
    }))
}

/// Maps to: CC `executeTeammateIdleHooks()` (hooks.ts:3709-3729).
/// If a hook blocks (exit code 2), the teammate should continue working.
pub async fn execute_teammate_idle_hooks(
    config: &RegisteredHooks,
    teammate_name: &str,
    team_name: &str,
    permission_mode: Option<&str>,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    // Maps to: CC `hooks.ts:1978-1999`. `executeTeammateIdleHooks`
    // (`:3723-3728`) passes no `matchQuery`.
    if crate::services::hooks::should_skip_hook_execution(HookEvent::TeammateIdle, "") {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, HookEvent::TeammateIdle, "", None);
    if matched.is_empty() {
        return Vec::new();
    }

    let input_str = teammate_base_input(permission_mode)
        .set("hook_event_name", "TeammateIdle")
        .set("teammate_name", teammate_name)
        .set("team_name", team_name)
        .build()
        .to_string();

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
                let start = Instant::now();
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

                let mut result = match parse_hook_output(&exec_result.stdout) {
                    ParsedHookOutput::Json(json) => {
                        process_hook_json_output(&json, &command.command)
                    }
                    _ => HookResult::default(),
                };
                attach_hook_execution_metadata(&mut result, &command.command, start, &exec_result);
                result
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

/// Maps to: CC `executeSubagentStartHooks()` (hooks.ts:3932-3952).
pub async fn execute_subagent_start_hooks(
    config: &RegisteredHooks,
    agent_id: &str,
    agent_type: &str,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    // Maps to: CC `hooks.ts:1978-1999`. `executeSubagentStartHooks`
    // (`:3945-3951`) passes `matchQuery: agentType`.
    if crate::services::hooks::should_skip_hook_execution(HookEvent::SubagentStart, agent_type) {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, HookEvent::SubagentStart, agent_type, None);
    if matched.is_empty() {
        return Vec::new();
    }

    // CC `createBaseHookInput(undefined)` (`hooks.ts:3939`) — no permissionMode.
    let input_str = teammate_base_input(None)
        .set("hook_event_name", "SubagentStart")
        .set("agent_id", agent_id)
        .set("agent_type", agent_type)
        .build()
        .to_string();

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
                let start = Instant::now();
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

                let mut result = match parse_hook_output(&exec_result.stdout) {
                    ParsedHookOutput::Json(json) => {
                        process_hook_json_output(&json, &command.command)
                    }
                    _ => HookResult::default(),
                };
                attach_hook_execution_metadata(&mut result, &command.command, start, &exec_result);
                result
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

/// Maps to: CC `executeStopHooks(...)` when `subagentId` is present: it uses
/// hook event `SubagentStop`, includes the agent transcript path, agent type,
/// and last assistant text.
pub async fn execute_subagent_stop_hooks(
    config: &RegisteredHooks,
    agent_id: &str,
    agent_type: &str,
    agent_transcript_path: &str,
    last_assistant_message: Option<&str>,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    // Maps to: CC `hooks.ts:1978-1999`, reached from `executeStopHooks`'
    // subagent branch (`:3688-3696`), whose own comment at `:3687` reads
    // "Trust check is now centralized in executeHooks()". CC's second historical
    // vulnerability (`:282`) is exactly this event: "SubagentStop hooks
    // executing when subagent completes before trust".
    //
    // Seam: CC passes no `matchQuery` here, while this port matches on
    // `agent_type`; the gate is handed the port's own query so the `--debug`
    // line names the same thing the matcher used.
    if crate::services::hooks::should_skip_hook_execution(HookEvent::SubagentStop, agent_type) {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, HookEvent::SubagentStop, agent_type, None);
    if matched.is_empty() {
        return Vec::new();
    }

    // Maps to: CC `hooks.ts:3670-3679` and `SubagentStopHookInputSchema`
    // (`coreSchemas.ts:550-567`). `stop_hook_active` is REQUIRED there and was
    // missing; CC's only producer of a SubagentStop input is `executeStopHooks`,
    // whose caller supplies `stopHookActive ?? false`
    // (`query/stopHooks.ts:184`). This port reaches SubagentStop from the agent
    // runner instead, which has no re-entrant Stop pass, so the value CC would
    // compute there is `false`.
    //
    // Seam: CC's base here is `createBaseHookInput(permissionMode)`; the
    // runner-side caller (`tools/agent_tool/run_agent.rs`) does not forward one,
    // so `permission_mode` is absent.
    let input_str = teammate_base_input(None)
        .set("hook_event_name", "SubagentStop")
        .set("stop_hook_active", false)
        .set("agent_id", agent_id)
        .set("agent_transcript_path", agent_transcript_path)
        .set("agent_type", agent_type)
        .set_optional("last_assistant_message", last_assistant_message)
        .build()
        .to_string();

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
                let start = Instant::now();
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

                let mut result = match parse_hook_output(&exec_result.stdout) {
                    ParsedHookOutput::Json(json) => {
                        process_hook_json_output(&json, &command.command)
                    }
                    _ => HookResult::default(),
                };
                attach_hook_execution_metadata(&mut result, &command.command, start, &exec_result);
                result
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

fn attach_hook_execution_metadata(
    result: &mut HookResult,
    command: &str,
    start: Instant,
    exec_result: &CommandExecResult,
) {
    result.command = Some(command.to_string());
    result.duration_ms = Some(start.elapsed().as_millis() as u64);
    result.stdout = Some(exec_result.stdout.clone());
    result.stderr = Some(exec_result.stderr.clone());
    if exec_result.status != 0 && result.outcome == HookOutcome::Success {
        result.outcome = HookOutcome::NonBlockingError;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::hooks::HooksConfig;
    use crate::services::hooks::test_support::{SessionTrustGuard, registered_config};

    #[tokio::test]
    async fn execute_subagent_start_hooks_sends_official_input_shape() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "SubagentStart": [{
                "matcher": "general-purpose",
                "hooks": [{ "command": "cat", "timeout": 5 }]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);

        let results =
            execute_subagent_start_hooks(&config, "agent-1", "general-purpose", Vec::new()).await;

        assert_eq!(results.len(), 1);
        let stdout = results[0].stdout.as_deref().unwrap_or_default();
        assert!(stdout.contains("\"hook_event_name\":\"SubagentStart\""));
        assert!(stdout.contains("\"agent_id\":\"agent-1\""));
        assert!(stdout.contains("\"agent_type\":\"general-purpose\""));
    }

    #[tokio::test]
    async fn execute_subagent_stop_hooks_sends_official_input_shape() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "SubagentStop": [{
                "matcher": "general-purpose",
                "hooks": [{ "command": "cat", "timeout": 5 }]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);

        let results = execute_subagent_stop_hooks(
            &config,
            "agent-1",
            "general-purpose",
            "/tmp/agent-agent-1.jsonl",
            Some("final answer"),
            Vec::new(),
        )
        .await;

        assert_eq!(results.len(), 1);
        let stdout = results[0].stdout.as_deref().unwrap_or_default();
        assert!(stdout.contains("\"hook_event_name\":\"SubagentStop\""));
        assert!(stdout.contains("\"agent_id\":\"agent-1\""));
        assert!(stdout.contains("\"agent_transcript_path\":\"/tmp/agent-agent-1.jsonl\""));
        assert!(stdout.contains("\"last_assistant_message\":\"final answer\""));
    }
}
