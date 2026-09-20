//! Lifecycle hook execution: Stop, Notification, SessionStart, SessionEnd, Setup.
//! Maps to: CC utils/hooks.ts:3570-3922.
//!
//! CC splits these five builders across its TWO executors, and the split is
//! what [`Executor`] records: Stop (`:3688`), SessionStart (`:3884`) and Setup
//! (`:3914`) end in `yield* executeHooks({…})`, while Notification (`:3587`),
//! StopFailure (`:3621`) and SessionEnd (`:4119`) `await
//! executeHooksOutsideREPL({…})`. Both mint the `toolUseID` a callback hook
//! receives; neither can hand it `undefined`.

use super::exec::{SESSION_END_TIMEOUT_MS, exec_command_hook};
use super::matching::get_matching_hooks;
use super::parsing::{ParsedHookOutput, parse_hook_output, process_hook_json_output};
use super::{HookContext, HookEvent, HookInputBuilder, HookOutcome, HookResult, RegisteredHooks};
use std::time::{Duration, Instant};

const TOOL_HOOK_TIMEOUT_MS: u64 = 30_000;

/// The progress a hook batch reports while it runs.
///
/// Maps to: CC `executeHooks` (hooks.ts:2094-2116) — "Yield progress messages
/// for each hook before execution" — whose payload is `types/hooks.ts:234-241`
/// `HookProgress` inside a progress message carrying `toolUseID`. CC's consumer
/// PULLS an `AsyncGenerator`; this executor returns `Vec<HookResult>`, so the
/// same notifications are PUSHED through a sink the caller supplies.
///
/// `Completed`/`Finished` have no yield of their own in CC: its consumer counts
/// hooks off the resolved attachments that arrive next
/// (`query/stopHooks.ts:217-255`) and stops rendering the suffix when the
/// generator is exhausted. The port's spinner consumes counted events instead
/// (`query.rs#StopHookProgressEvent`), so the executor states both explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookProgress {
    Started {
        tool_use_id: String,
        hook_event: String,
        command: String,
        status_message: Option<String>,
        total: usize,
    },
    Completed {
        tool_use_id: String,
        hook_event: String,
    },
    Finished {
        tool_use_id: String,
    },
}

/// Push side of CC's `AsyncGenerator<AggregatedHookResult>`.
///
/// Returns `false` once the consumer is gone, which ends the batch the way a
/// dropped consumer ends CC's generator.
pub type HookProgressSink =
    dyn Fn(HookProgress) -> futures::future::BoxFuture<'static, bool> + Send + Sync;

/// Which of CC's two executors an event's family reaches.
///
/// The distinction is not cosmetic: it decides where the `toolUseID` a callback
/// hook receives is minted, and whether progress is yielded at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Executor {
    /// CC `executeHooks` (hooks.ts:1955-2116): ONE `toolUseID: randomUUID()`
    /// per batch, minted by the builder — Stop (`:3690`), SessionStart
    /// (`:3886`), Setup (`:3916`) — and shared by every hook in the batch and
    /// by its progress messages (`:2110-2111`). This is the family that yields
    /// progress.
    InRepl,
    /// CC `executeHooksOutsideREPL` (hooks.ts:3002-3380): yields nothing, and
    /// mints a FRESH `randomUUID()` per callback hook (`:3095`). Notification
    /// (`:3587`), StopFailure (`:3621`) and SessionEnd (`:4119`) are its
    /// lifecycle members.
    OutsideRepl,
}

/// The arguments CC's executors take as one destructured object
/// (`executeHooks({hookInput, toolUseID, matchQuery, signal, …})`,
/// hooks.ts:1955-1977).
struct EventExecution<'a> {
    event: HookEvent,
    match_query: &'a str,
    input: &'a serde_json::Value,
    base_env: Vec<(String, String)>,
    executor: Executor,
    /// CC `signal?: AbortSignal`. Only the three builders CC gives one can
    /// supply it: Stop (`:3641`), SessionStart (`:3872`), Setup (`:3904`).
    /// `executeNotificationHooks` (`:3570-3577`) and `executeStopFailureHooks`
    /// (`:3594-3598`) have no signal parameter at all, so they pass `None` —
    /// CC's `undefined`.
    abort: Option<&'a crate::tool::AbortController>,
    /// `None` for every builder whose port-side caller collects results instead
    /// of streaming rows. CC yields progress for the whole `executeHooks`
    /// family, so SessionStart/Setup passing `None` is a consumer-side seam,
    /// not a claim that CC is silent there; Stop is the one lifecycle event
    /// with a live consumer (the REPL spinner).
    progress: Option<&'a HookProgressSink>,
}

/// The `...createBaseHookInput(...)` spread every lifecycle builder opens with.
///
/// Seam: CC's `agentInfo` argument is only ever `toolUseContext` (and only for
/// PreToolUse/PostToolUse/PostToolUseFailure/PermissionDenied/StopFailure/
/// PermissionRequest). Every lifecycle event this module owns passes NO agent
/// info (`hooks.ts:3580`, `:3672`, `:3681`, `:3877`, `:3909`, `:4114`), so
/// `agent_id` is correctly absent and `agent_type` is set only where the event
/// itself declares one.
fn lifecycle_base_input(
    permission_mode: Option<&str>,
    session_id: Option<&str>,
) -> HookInputBuilder {
    HookInputBuilder::from_base(super::create_base_hook_input_object(&HookContext {
        session_id: session_id.unwrap_or_default().to_string(),
        permission_mode: permission_mode.map(str::to_string),
        ..Default::default()
    }))
}

/// Maps to: CC `executeSessionStartHooks()` (hooks.ts:3867-3892) and
/// `SessionStartHookInputSchema` (`coreSchemas.ts:493-502`).
pub async fn execute_session_start_hooks(
    config: &RegisteredHooks,
    source: &str,
    session_id: Option<&str>,
    agent_type: Option<&str>,
    model: Option<&str>,
    base_env: Vec<(String, String)>,
    abort: Option<&crate::tool::AbortController>,
) -> Vec<HookResult> {
    // CC `createBaseHookInput(undefined, sessionId)`: the explicit session id
    // also drives `transcript_path`, and it is the ONLY builder that overrides
    // it (`hooks.ts:3877`).
    let input = lifecycle_base_input(None, session_id)
        .set("hook_event_name", "SessionStart")
        .set("source", source)
        .set_optional("agent_type", agent_type)
        .set_optional("model", model)
        .build();
    execute_event(
        config,
        EventExecution {
            event: HookEvent::SessionStart,
            match_query: source,
            input: &input,
            base_env,
            executor: Executor::InRepl,
            abort,
            progress: None,
        },
    )
    .await
}

/// Maps to: CC `executeSetupHooks()` (hooks.ts:3902-3922).
pub async fn execute_setup_hooks(
    config: &RegisteredHooks,
    trigger: &str,
    base_env: Vec<(String, String)>,
    abort: Option<&crate::tool::AbortController>,
) -> Vec<HookResult> {
    let input = lifecycle_base_input(None, None)
        .set("hook_event_name", "Setup")
        .set("trigger", trigger)
        .build();
    execute_event(
        config,
        EventExecution {
            event: HookEvent::Setup,
            match_query: trigger,
            input: &input,
            base_env,
            executor: Executor::InRepl,
            abort,
            progress: None,
        },
    )
    .await
}

/// Maps to: CC `executeStopHooks()` non-subagent branch (hooks.ts:3680-3685)
/// and `StopHookInputSchema` (`coreSchemas.ts:513-527`).
///
/// `stop_hook_active` is REQUIRED by that schema and was missing here. CC's
/// caller supplies `stopHookActive ?? false` (`query/stopHooks.ts:184`) so a
/// hook can tell a re-entrant Stop pass from the first one.
///
/// This is the ONLY Stop executor, as in CC: `ast-grep run -p 'async function*
/// executeStopHooks' -l ts` returns one hit, whose body is payload construction
/// plus `yield* executeHooks({…})`. `query/stop_hooks.rs` used to carry a
/// second, progress-emitting copy of the loop below; progress is a parameter
/// here because in CC it belongs to the shared executor (`hooks.ts:2094-2116`),
/// not to a Stop-specific variant.
///
/// The empty `match_query` is CC's: `executeStopHooks` passes no `matchQuery`
/// (`:3688-3696`), so `hookName` is the bare event name (`:1987`).
pub async fn execute_stop_hooks(
    config: &RegisteredHooks,
    permission_mode: Option<&str>,
    stop_hook_active: bool,
    last_assistant_message: Option<&str>,
    base_env: Vec<(String, String)>,
    abort: Option<&crate::tool::AbortController>,
    progress: Option<&HookProgressSink>,
    subagent_id: Option<&str>,
    agent_type: Option<&str>,
) -> Vec<HookResult> {
    // CC hooks.ts:3653,3670-3685: a subagent stops with SubagentStop, never Stop.
    let event = if subagent_id.is_some_and(|id| !id.is_empty()) {
        HookEvent::SubagentStop
    } else {
        HookEvent::Stop
    };
    let mut input = lifecycle_base_input(permission_mode, None)
        .set("hook_event_name", event.as_str())
        .set("stop_hook_active", stop_hook_active)
        .set_optional("last_assistant_message", last_assistant_message)
        .build();
    if event == HookEvent::SubagentStop {
        input["agent_id"] = subagent_id.unwrap().into();
        input["agent_transcript_path"] =
            crate::utils::session_storage::get_agent_transcript_path(subagent_id.unwrap())
                .display()
                .to_string()
                .into();
        input["agent_type"] = agent_type.unwrap_or("").into();
    }
    execute_event(
        config,
        EventExecution {
            event,
            match_query: if event == HookEvent::SubagentStop {
                agent_type.unwrap_or("")
            } else {
                ""
            },
            input: &input,
            base_env,
            executor: Executor::InRepl,
            abort,
            progress,
        },
    )
    .await
}

/// Maps to: CC `executeStopFailureHooks()` (hooks.ts:3594-3627).
///
/// Seam: CC threads `toolUseContext` into the base here (`:3614`), which is the
/// one lifecycle builder that carries `agent_id`/`agent_type`. The port's
/// caller (`query.rs#execute_stop_failure_hooks_for_api_error`) holds only a
/// `ToolPermissionContext`, so those two stay absent — the main-thread case,
/// where CC also sends neither.
pub async fn execute_stop_failure_hooks(
    config: &RegisteredHooks,
    error: &str,
    error_details: Option<&str>,
    last_assistant_message: Option<&str>,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    let input = lifecycle_base_input(None, None)
        .set("hook_event_name", "StopFailure")
        .set("error", error)
        .set_optional("error_details", error_details)
        .set_optional("last_assistant_message", last_assistant_message)
        .build();
    execute_event(
        config,
        EventExecution {
            event: HookEvent::StopFailure,
            match_query: error,
            input: &input,
            base_env,
            executor: Executor::OutsideRepl,
            // CC `executeStopFailureHooks(lastMessage, toolUseContext,
            // timeoutMs)` (`hooks.ts:3594-3598`) has no signal parameter, so
            // `executeHooksOutsideREPL` sees `signal: undefined` here.
            abort: None,
            progress: None,
        },
    )
    .await
}

/// Maps to: CC `executeNotificationHooks()` (hooks.ts:3570-3592).
/// Fire-and-forget — results are not consumed by the caller.
pub async fn execute_notification_hooks(
    config: &RegisteredHooks,
    message: &str,
    notification_type: &str,
    title: Option<&str>,
    base_env: Vec<(String, String)>,
) {
    // CC's literal order is `message, title, notification_type`
    // (`hooks.ts:3582-3584`); `title` is `z.string().optional()`
    // (`coreSchemas.ts:478`) so an absent title sends no key.
    let input = lifecycle_base_input(None, None)
        .set("hook_event_name", "Notification")
        .set("message", message)
        .set_optional("title", title)
        .set("notification_type", notification_type)
        .build();
    let _ = execute_event(
        config,
        EventExecution {
            event: HookEvent::Notification,
            match_query: notification_type,
            input: &input,
            base_env,
            executor: Executor::OutsideRepl,
            // CC `executeNotificationHooks(notificationData, timeoutMs)`
            // (`hooks.ts:3570-3577`) has no signal parameter either.
            abort: None,
            progress: None,
        },
    )
    .await;
}

/// Maps to: CC `executeSessionEndHooks(reason, ...)` (hooks.ts:4097-4157).
/// Uses shorter timeout (SESSION_END_TIMEOUT_MS) since the process is exiting
/// or the conversation is being cleared.
pub async fn execute_session_end_hooks(
    config: &RegisteredHooks,
    reason: &str,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    // Maps to: CC `hooks.ts:3016-3036` reached through
    // `executeHooksOutsideREPL({hookInput, matchQuery: reason, …})`
    // (`:4119-4124`). SessionEnd does not share `execute_event`'s loop (it has
    // its own, for the shorter shutdown timeout), so it needs the gate in its
    // own right — the historical vulnerability CC names at `:281` is literally
    // "SessionEnd hooks executing when user declines trust dialog".
    if crate::services::hooks::should_skip_hook_execution(HookEvent::SessionEnd, reason) {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, HookEvent::SessionEnd, reason, None);
    if matched.is_empty() {
        return Vec::new();
    }

    // Maps to: CC `hooks.ts:4113-4117` — base spread plus `reason`.
    // `session_id` is a BASE key (`coreSchemas.ts:389`), which this builder used
    // to write by hand while omitting `transcript_path` and `cwd`.
    let input_str = lifecycle_base_input(None, None)
        .set("hook_event_name", "SessionEnd")
        .set("reason", reason)
        .build()
        .to_string();
    let mut results = Vec::new();
    for hook in &matched {
        // CC hooks.ts:2147 — callback hooks resolve via the SDK consumer.
        let result = match &hook.hook {
            crate::schemas::hooks::RegisteredHook::Callback(callback) => {
                // SessionEnd reaches CC's `executeHooksOutsideREPL` (`:4119`),
                // which mints a fresh `randomUUID()` per callback hook
                // (`:3095`). Old shape: `None` — the same null the shared
                // `execute_event` used to hand every other lifecycle event.
                let json = super::exec::exec_callback_hook(
                    callback,
                    &input_str,
                    Some(uuid::Uuid::new_v4().to_string()),
                )
                .await;
                process_hook_json_output(&json, "callback")
            }
            crate::schemas::hooks::RegisteredHook::Command(command) => {
                let timeout = command
                    .timeout
                    .map(|t| t * 1000)
                    .unwrap_or(SESSION_END_TIMEOUT_MS);
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
        results.push(result);
    }
    results
}

/// The one loop every lifecycle event below runs through, standing in for both
/// of CC's executors (see [`Executor`]).
///
/// Maps to: CC `executeHooks` (hooks.ts:1955-2116, :2143-2700) and
/// `executeHooksOutsideREPL` (`:3002-3380`).
async fn execute_event(config: &RegisteredHooks, execution: EventExecution<'_>) -> Vec<HookResult> {
    let EventExecution {
        event,
        match_query,
        input,
        base_env,
        executor,
        abort,
        progress,
    } = execution;

    // Maps to: CC `hooks.ts:1978-1999` / `:3016-3036`. SessionStart, Setup,
    // Stop, StopFailure and Notification all reach `execCommandHook` through
    // this one loop, and none of them was gated before — an enterprise
    // `disableAllHooks` still ran every one of them.
    if crate::services::hooks::should_skip_hook_execution(event, match_query) {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, event, match_query, None);
    if matched.is_empty() {
        return Vec::new();
    }

    // Maps to: CC `hooks.ts:2015-2017` and `:3051-3053` — both executors bail
    // AFTER matching when the signal is already aborted.
    if abort.is_some_and(crate::tool::AbortController::is_aborted) {
        return Vec::new();
    }

    // CC's `toolUseID`, which `executeHookCallback` types as a non-optional
    // `string` (`hooks.ts:4840-4856`). Old shape: `None` — an SDK callback hook
    // on Stop, SessionStart, Setup, StopFailure or Notification received `null`
    // where CC guarantees an id. `Executor` decides the scope: one id for the
    // whole batch here, a fresh one per callback hook below.
    let batch_tool_use_id = uuid::Uuid::new_v4().to_string();

    // Maps to: CC `hooks.ts:2094-2116` — "Yield progress messages for each hook
    // before execution". The position is load-bearing: every matched hook is
    // announced BEFORE the first one runs, so a consumer that counts them knows
    // the batch size up front.
    if let Some(progress) = progress {
        let total = matched.len();
        for hook in &matched {
            let (command, status_message) = hook_progress_text(&hook.hook);
            let delivered = progress(HookProgress::Started {
                tool_use_id: batch_tool_use_id.clone(),
                hook_event: event.as_str().to_string(),
                command,
                status_message,
                total,
            })
            .await;
            if !delivered {
                return Vec::new();
            }
        }
    }

    let input_str = input.to_string();
    let mut results = Vec::new();
    for hook in &matched {
        // CC hooks.ts:2147 — callback hooks resolve via the SDK consumer.
        let result = match &hook.hook {
            crate::schemas::hooks::RegisteredHook::Callback(callback) => {
                let tool_use_id = match executor {
                    Executor::InRepl => batch_tool_use_id.clone(),
                    Executor::OutsideRepl => uuid::Uuid::new_v4().to_string(),
                };
                let json =
                    super::exec::exec_callback_hook(callback, &input_str, Some(tool_use_id)).await;
                process_hook_json_output(&json, "callback")
            }
            crate::schemas::hooks::RegisteredHook::Command(command) => {
                let timeout = command.timeout.unwrap_or(TOOL_HOOK_TIMEOUT_MS / 1000) * 1000;
                let start = Instant::now();
                // Seam: CC combines `signal` with the per-hook timeout and
                // passes it to `execCommandHook` (`:2196`, `:2453`), so an
                // abort KILLS a running hook and reports it as `cancelled`
                // (`:2473-2496`). This port stops starting further hooks
                // instead; wiring the controller in without the `cancelled`
                // outcome would report a killed hook as a non-blocking error.
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
                    ParsedHookOutput::PlainText(text) => HookResult {
                        system_message: Some(text),
                        ..Default::default()
                    },
                    ParsedHookOutput::ValidationError { error, .. } => HookResult {
                        outcome: HookOutcome::NonBlockingError,
                        system_message: Some(format!("Hook output error: {error}")),
                        ..Default::default()
                    },
                    ParsedHookOutput::Empty => HookResult::default(),
                };
                attach_hook_execution_metadata(&mut result, &command.command, start, &exec_result);
                result
            }
        };

        let should_stop = result.prevent_continuation || result.outcome == HookOutcome::Blocking;
        results.push(result);

        if let Some(progress) = progress {
            let delivered = progress(HookProgress::Completed {
                tool_use_id: batch_tool_use_id.clone(),
                hook_event: event.as_str().to_string(),
            })
            .await;
            if !delivered {
                return results;
            }
        }

        if should_stop {
            break;
        }
        // CC's consumer checks the signal at the end of every iteration of its
        // `for await (const result of generator)` loop (`stopHooks.ts:283`).
        // CC's own hooks are already in flight by then (`:2143` maps them all
        // at once) and get killed through their combined signal; this loop is
        // sequential, so the same interrupt lands as "run no further hooks".
        if abort.is_some_and(crate::tool::AbortController::is_aborted) {
            break;
        }
    }

    if let Some(progress) = progress {
        progress(HookProgress::Finished {
            tool_use_id: batch_tool_use_id,
        })
        .await;
    }

    results
}

/// The `command` and `statusMessage` fields of a progress message.
///
/// Maps to: CC `hooks.ts:2103-2108`. Deviation: CC's `command` is
/// `getHookDisplayText(hook)`, which returns the status message when one is set
/// (`utils/hooks/hooksSettings.ts:72-73`) and `'callback'` for callback hooks
/// (`:85-86`); the port sends the raw command and lets the consumer prefer the
/// status message (`repl.rs#stop_hook_spinner_suffix`), which renders the same.
fn hook_progress_text(hook: &crate::schemas::hooks::RegisteredHook) -> (String, Option<String>) {
    match hook {
        crate::schemas::hooks::RegisteredHook::Command(command) => {
            (command.command.clone(), command.status.clone())
        }
        // `statusMessage` is a command-hook field; callbacks have no command
        // string (CC `hooks.ts:4885` `command: 'callback'`).
        crate::schemas::hooks::RegisteredHook::Callback(_) => ("callback".to_string(), None),
    }
}

fn attach_hook_execution_metadata(
    result: &mut HookResult,
    command: &str,
    start: Instant,
    exec_result: &super::CommandExecResult,
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
    async fn subagent_stop_matches_official_event_selection() {
        // CC hooks.ts:3653-3696 selects SubagentStop and matches agentType.
        let _trust = SessionTrustGuard::accepted();
        let log: CallbackLog = Default::default();
        let config = callback_config(
            &[
                (HookEvent::Stop, None),
                (HookEvent::SubagentStop, Some("verifier")),
            ],
            &log,
            1,
        );
        execute_stop_hooks(
            &config,
            None,
            false,
            None,
            Vec::new(),
            None,
            None,
            Some("hook-agent-test"),
            Some("verifier"),
        )
        .await;
        let seen = logged(&log);
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, "SubagentStop");
    }

    #[tokio::test]
    async fn execute_notification_hooks_sends_official_input_shape() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "Notification": [{
                "matcher": "elicitation_response",
                "hooks": [{
                    "command": "cat",
                    "timeout": 5
                }]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);

        execute_notification_hooks(
            &config,
            "Elicitation response for server \"docs\": accept",
            "elicitation_response",
            None,
            Vec::new(),
        )
        .await;

        let results = execute_event(
            &config,
            EventExecution {
                event: HookEvent::Notification,
                match_query: "elicitation_response",
                input: &serde_json::json!({
                    "hook_event_name": "Notification",
                    "message": "Elicitation response for server \"docs\": accept",
                    "title": null,
                    "notification_type": "elicitation_response",
                }),
                base_env: Vec::new(),
                executor: Executor::OutsideRepl,
                abort: None,
                progress: None,
            },
        )
        .await;
        let stdout = results[0].stdout.as_deref().unwrap_or_default();
        assert!(stdout.contains("\"hook_event_name\":\"Notification\""));
        assert!(stdout.contains("\"notification_type\":\"elicitation_response\""));
        assert!(stdout.contains("Elicitation response for server"));
    }

    #[tokio::test]
    async fn execute_session_start_hooks_sends_official_resume_input_shape() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "SessionStart": [{
                "matcher": "resume",
                "hooks": [{"command": "cat", "timeout": 5}]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);

        let results = execute_session_start_hooks(
            &config,
            "resume",
            Some("target-session"),
            Some("reviewer"),
            Some("claude-sonnet"),
            Vec::new(),
            None,
        )
        .await;

        assert_eq!(results.len(), 1);
        let stdout = results[0].stdout.as_deref().unwrap_or_default();
        assert!(stdout.contains("\"hook_event_name\":\"SessionStart\""));
        assert!(stdout.contains("\"source\":\"resume\""));
        assert!(stdout.contains("\"session_id\":\"target-session\""));
        assert!(stdout.contains("\"agent_type\":\"reviewer\""));
        assert!(stdout.contains("\"model\":\"claude-sonnet\""));
    }

    #[tokio::test]
    async fn execute_session_end_hooks_sends_official_resume_reason() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "SessionEnd": [{
                "matcher": "*",
                "hooks": [{"command": "cat", "timeout": 5}]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);

        let results = execute_session_end_hooks(&config, "resume", Vec::new()).await;

        assert_eq!(results.len(), 1);
        let stdout = results[0].stdout.as_deref().unwrap_or_default();
        assert!(stdout.contains("\"hook_event_name\":\"SessionEnd\""));
        assert!(stdout.contains("\"reason\":\"resume\""));
        assert!(stdout.contains("\"session_id\":"));
    }

    #[tokio::test]
    async fn execute_stop_failure_hooks_sends_official_input_shape() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "StopFailure": [{
                "matcher": "*",
                "hooks": [{
                    "command": "cat",
                    "timeout": 5
                }]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);

        let results = execute_stop_failure_hooks(
            &config,
            "API boom",
            Some("details"),
            Some("partial assistant"),
            Vec::new(),
        )
        .await;

        assert_eq!(results.len(), 1);
        let stdout = results[0].stdout.as_deref().unwrap_or_default();
        assert!(stdout.contains("\"hook_event_name\":\"StopFailure\""));
        assert!(stdout.contains("\"error\":\"API boom\""));
        assert!(stdout.contains("\"error_details\":\"details\""));
        assert!(stdout.contains("\"last_assistant_message\":\"partial assistant\""));
    }

    /// What an SDK callback hook was handed: `(hook_event_name, tool_use_id)`.
    type CallbackLog = std::sync::Arc<std::sync::Mutex<Vec<(String, Option<String>)>>>;

    /// `count` SDK callback hooks that record the arguments they receive.
    /// Maps to: CC `HookCallback.callback(hookInput, toolUseID, …)`
    /// (`hooks.ts:4864-4870`, `:3096-3101`).
    fn recording_callbacks(
        log: &CallbackLog,
        count: usize,
    ) -> Vec<crate::schemas::hooks::RegisteredHook> {
        (0..count)
            .map(|_| {
                let log = log.clone();
                crate::schemas::hooks::RegisteredHook::Callback(
                    crate::schemas::hooks::HookCallback {
                        callback: std::sync::Arc::new(
                            move |input: serde_json::Value, tool_use_id| {
                                let log = log.clone();
                                Box::pin(async move {
                                    let event = input
                                        .get("hook_event_name")
                                        .and_then(serde_json::Value::as_str)
                                        .unwrap_or_default()
                                        .to_string();
                                    log.lock()
                                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                                        .push((event, tool_use_id));
                                    serde_json::json!({})
                                })
                            },
                        ),
                        timeout: Some(5),
                    },
                )
            })
            .collect()
    }

    fn callback_config(
        events: &[(HookEvent, Option<&str>)],
        log: &CallbackLog,
        count: usize,
    ) -> RegisteredHooks {
        events
            .iter()
            .map(|(event, matcher)| {
                (
                    event.as_str().to_string(),
                    vec![crate::schemas::hooks::RegisteredHookMatcher {
                        matcher: matcher.map(str::to_string),
                        hooks: recording_callbacks(log, count),
                        plugin_root: None,
                        plugin_name: None,
                        plugin_id: None,
                    }],
                )
            })
            .collect()
    }

    fn logged(log: &CallbackLog) -> Vec<(String, Option<String>)> {
        log.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Maps to: CC `executeHookCallback({toolUseID, …})` (hooks.ts:4840-4856),
    /// which types `toolUseID: string` — NON-optional — and is reached only
    /// with a `randomUUID()`: `executeHooks` takes one as a required parameter
    /// (`:1965`) and every lifecycle builder supplies it (`:3690` Stop, `:3886`
    /// SessionStart, `:3916` Setup), while `executeHooksOutsideREPL` mints its
    /// own per callback hook (`:3095`) for Notification, StopFailure and
    /// SessionEnd. No CC path hands a hook `undefined`.
    ///
    /// Old shape: the shared `execute_event` passed `None` to
    /// `exec_callback_hook` (lifecycle.rs:233), and `execute_session_end_hooks`
    /// passed `None` in its own loop, so an SDK callback hook on any of these
    /// SIX events received `null`. A wrong value, not a hang — the callback ran
    /// and read a null id — so this fails on the `Some` assertion below.
    #[tokio::test]
    async fn every_lifecycle_event_hands_its_callback_hook_a_tool_use_id() {
        let _trust = SessionTrustGuard::accepted();
        let log: CallbackLog = Default::default();
        let config = callback_config(
            &[
                (HookEvent::Stop, None),
                (HookEvent::SessionStart, Some("resume")),
                (HookEvent::Setup, Some("init")),
                (HookEvent::StopFailure, Some("*")),
                (HookEvent::Notification, Some("*")),
                (HookEvent::SessionEnd, Some("*")),
            ],
            &log,
            1,
        );

        execute_stop_hooks(
            &config,
            Some("default"),
            false,
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
        )
        .await;
        execute_session_start_hooks(&config, "resume", None, None, None, Vec::new(), None).await;
        execute_setup_hooks(&config, "init", Vec::new(), None).await;
        execute_stop_failure_hooks(&config, "API boom", None, None, Vec::new()).await;
        execute_notification_hooks(&config, "hello", "permission", None, Vec::new()).await;
        execute_session_end_hooks(&config, "clear", Vec::new()).await;

        let seen = logged(&log);
        assert_eq!(
            seen.iter()
                .map(|(event, _)| event.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Stop",
                "SessionStart",
                "Setup",
                "StopFailure",
                "Notification",
                "SessionEnd",
            ],
            "each builder has to reach its callback hook, or the fixture is wrong"
        );
        for (event, tool_use_id) in seen {
            let tool_use_id = tool_use_id
                .unwrap_or_else(|| panic!("{event} handed its callback hook a null tool_use_id"));
            assert!(
                uuid::Uuid::parse_str(&tool_use_id).is_ok(),
                "{event} must supply CC's randomUUID(), got {tool_use_id:?}"
            );
        }
    }

    /// Maps to: the two executors' differing `toolUseID` SCOPE. `executeHooks`
    /// mints one per BATCH — the builder passes a single `randomUUID()`
    /// (`hooks.ts:3690`) that every hook in the batch and every progress
    /// message shares (`:2110-2111`, `:2154`) — while `executeHooksOutsideREPL`
    /// mints one per callback hook INSIDE its `matchingHooks.map` (`:3095`).
    #[tokio::test]
    async fn the_tool_use_id_is_shared_per_batch_only_in_the_execute_hooks_family() {
        let _trust = SessionTrustGuard::accepted();

        let stop_log: CallbackLog = Default::default();
        let stop_config = callback_config(&[(HookEvent::Stop, None)], &stop_log, 2);
        execute_stop_hooks(
            &stop_config,
            Some("default"),
            false,
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
        )
        .await;
        let stop_ids = logged(&stop_log)
            .into_iter()
            .map(|(_, tool_use_id)| tool_use_id)
            .collect::<Vec<_>>();
        assert_eq!(stop_ids.len(), 2);
        assert_eq!(
            stop_ids[0], stop_ids[1],
            "executeStopHooks passes ONE randomUUID() to executeHooks (hooks.ts:3690)"
        );

        let notification_log: CallbackLog = Default::default();
        let notification_config = callback_config(
            &[(HookEvent::Notification, Some("*"))],
            &notification_log,
            2,
        );
        execute_notification_hooks(
            &notification_config,
            "hello",
            "permission",
            None,
            Vec::new(),
        )
        .await;
        let notification_ids = logged(&notification_log)
            .into_iter()
            .map(|(_, tool_use_id)| tool_use_id)
            .collect::<Vec<_>>();
        assert_eq!(notification_ids.len(), 2);
        assert_ne!(
            notification_ids[0], notification_ids[1],
            "executeHooksOutsideREPL mints a fresh id per callback hook (hooks.ts:3095)"
        );
    }

    /// Maps to: CC `hooks.ts:2015-2017` (`executeHooks`) and `:3051-3053`
    /// (`executeHooksOutsideREPL`) — both bail on an already-aborted signal
    /// after matching, so NO hook runs.
    ///
    /// Old shape: `execute_event` had no abort check at all, so all five events
    /// ran their hooks under an aborted signal. Only the deleted
    /// `query/stop_hooks.rs` copy checked it, and only per iteration. Fails on
    /// the emptiness assertions, no hang.
    ///
    /// The signal reaches the loop only where CC has a parameter for it: Stop
    /// (`:3641`), SessionStart (`:3872`) and Setup (`:3904`). Notification
    /// (`:3570-3577`) and StopFailure (`:3594-3598`) take no signal in CC, so
    /// the second half drives the shared loop directly rather than inventing
    /// one for them.
    #[tokio::test]
    async fn an_aborted_signal_stops_every_lifecycle_family_before_any_hook_runs() {
        let _trust = SessionTrustGuard::accepted();
        let abort = crate::tool::AbortController::default();
        abort.abort();
        let log: CallbackLog = Default::default();
        let config = callback_config(
            &[
                (HookEvent::Stop, None),
                (HookEvent::SessionStart, Some("resume")),
                (HookEvent::Setup, Some("init")),
                (HookEvent::StopFailure, Some("*")),
                (HookEvent::Notification, Some("*")),
            ],
            &log,
            1,
        );

        assert!(
            execute_stop_hooks(
                &config,
                Some("default"),
                false,
                None,
                Vec::new(),
                Some(&abort),
                None,
                None,
                None
            )
            .await
            .is_empty()
        );
        assert!(
            execute_session_start_hooks(
                &config,
                "resume",
                None,
                None,
                None,
                Vec::new(),
                Some(&abort)
            )
            .await
            .is_empty()
        );
        assert!(
            execute_setup_hooks(&config, "init", Vec::new(), Some(&abort))
                .await
                .is_empty()
        );

        for (event, match_query, input) in [
            (
                HookEvent::StopFailure,
                "boom",
                serde_json::json!({"hook_event_name": "StopFailure", "error": "boom"}),
            ),
            (
                HookEvent::Notification,
                "permission",
                serde_json::json!({
                    "hook_event_name": "Notification",
                    "message": "hello",
                    "notification_type": "permission",
                }),
            ),
        ] {
            assert!(
                execute_event(
                    &config,
                    EventExecution {
                        event,
                        match_query,
                        input: &input,
                        base_env: Vec::new(),
                        executor: Executor::OutsideRepl,
                        abort: Some(&abort),
                        progress: None,
                    },
                )
                .await
                .is_empty()
            );
        }

        assert!(
            logged(&log).is_empty(),
            "an aborted signal must stop the loop before any hook runs"
        );
    }

    /// Maps to: CC `query/stopHooks.ts:283` — the consumer re-checks the signal
    /// at the END of every iteration over the generator. CC's own hooks are all
    /// in flight by then (`hooks.ts:2143` maps them at once, each with a
    /// combined abort signal); this loop is sequential, so the same interrupt
    /// lands as "start no further hooks".
    ///
    /// Old shape: this check existed ONLY in the deleted `query/stop_hooks.rs`
    /// copy, so the four events that shared `lifecycle::execute_event` kept
    /// running hooks after an interrupt. Fails on the count (2, not 1).
    #[tokio::test]
    async fn an_abort_raised_mid_batch_stops_the_remaining_lifecycle_hooks() {
        let _trust = SessionTrustGuard::accepted();
        let abort = crate::tool::AbortController::default();
        let log: CallbackLog = Default::default();
        let mut hooks = recording_callbacks(&log, 2);
        // The first hook aborts while the batch is running, exactly as a user's
        // ctrl-c does mid-Stop.
        let interrupting = abort.clone();
        let interrupting_log = log.clone();
        hooks[0] =
            crate::schemas::hooks::RegisteredHook::Callback(crate::schemas::hooks::HookCallback {
                callback: std::sync::Arc::new(move |_input, tool_use_id| {
                    let interrupting = interrupting.clone();
                    let interrupting_log = interrupting_log.clone();
                    Box::pin(async move {
                        interrupting_log
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .push(("Stop".to_string(), tool_use_id));
                        interrupting.abort();
                        serde_json::json!({})
                    })
                }),
                timeout: Some(5),
            });
        let config: RegisteredHooks = std::collections::HashMap::from([(
            HookEvent::Stop.as_str().to_string(),
            vec![crate::schemas::hooks::RegisteredHookMatcher {
                matcher: None,
                hooks,
                plugin_root: None,
                plugin_name: None,
                plugin_id: None,
            }],
        )]);

        let results = execute_stop_hooks(
            &config,
            Some("default"),
            false,
            None,
            Vec::new(),
            Some(&abort),
            None,
            None,
            None,
        )
        .await;

        assert_eq!(results.len(), 1, "the second hook must not run");
        assert_eq!(logged(&log).len(), 1);
    }

    /// Maps to: CC `hooks.ts:2094-2116` — "Yield progress messages for each
    /// hook before execution", a loop over ALL matched hooks that runs before
    /// the batch starts, so a consumer knows the size up front.
    ///
    /// Old shape: progress lived only in the `query/stop_hooks.rs` copy, so
    /// this executor emitted nothing at all — the assertions below would see an
    /// empty log.
    #[tokio::test]
    async fn progress_announces_every_matched_hook_before_the_first_one_runs() {
        let _trust = SessionTrustGuard::accepted();
        let config: HooksConfig = serde_json::from_value(serde_json::json!({
            "Stop": [{
                "matcher": "*",
                "hooks": [
                    {"command": "printf first", "timeout": 5, "status": "Cleaning up"},
                    {"command": "printf second", "timeout": 5}
                ]
            }]
        }))
        .unwrap();
        let config = registered_config(&config);
        let progress: std::sync::Arc<std::sync::Mutex<Vec<HookProgress>>> = Default::default();
        let sink = {
            let progress = progress.clone();
            move |event: HookProgress| {
                let progress = progress.clone();
                Box::pin(async move {
                    progress
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .push(event);
                    true
                }) as futures::future::BoxFuture<'static, bool>
            }
        };

        let results = execute_stop_hooks(
            &config,
            Some("default"),
            false,
            None,
            Vec::new(),
            None,
            Some(&sink),
            None,
            None,
        )
        .await;

        assert_eq!(results.len(), 2);
        let progress = progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        // Both `Started`s precede either `Completed`: CC announces the batch,
        // then runs it.
        assert!(matches!(
            progress.as_slice(),
            [
                HookProgress::Started { total: 2, status_message: Some(status), .. },
                HookProgress::Started { total: 2, .. },
                HookProgress::Completed { .. },
                HookProgress::Completed { .. },
                HookProgress::Finished { .. },
            ] if status == "Cleaning up"
        ));
        let ids = progress
            .iter()
            .map(|event| match event {
                HookProgress::Started { tool_use_id, .. }
                | HookProgress::Completed { tool_use_id, .. }
                | HookProgress::Finished { tool_use_id } => tool_use_id.clone(),
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            ids.len(),
            1,
            "CC's progress messages all carry the batch's one toolUseID (hooks.ts:2110-2111)"
        );
    }
}
