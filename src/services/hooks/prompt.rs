//! UserPromptSubmit hook execution.
//! Maps to: CC utils/hooks.ts:3826-3855 (executeUserPromptSubmitHooks).
//!
//! # Partially wired seam
//!
//! CC consumes this in `processUserInput.ts:182-262`, after
//! `processUserInputBase` and only when `shouldQuery` survived. Both halves of
//! that consumption now exist in `utils/process_user_input`: the shaping in
//! `apply_user_prompt_submit_hook_results`, and the awaiting in the outer
//! `process_user_input`, which calls this function.
//!
//! What is still missing is the *production* path into that outer layer, so a
//! configured `UserPromptSubmit` hook does not run yet. CC's whole submit chain
//! is async — `onSubmit` (`REPL.tsx:4241`) → `handlePromptSubmit` (`:120`) →
//! `processUserInput` — while the Rust chain is synchronous from the iocraft
//! `on_submit` callback down, and starts the query inline. Awaiting from there
//! would block the render thread; starting the query first would defeat
//! `blockingError` / `preventContinuation`, both of which must be able to stop
//! the turn. So `handle_prompt_submit` still calls `process_user_input_base`.
//! Closing this means making that chain async the way CC's already is
//! (TaskList #31 steps 2-3); `screens::repl`'s MCP prompt path already shows
//! the shape — `use_async_handler` around the same `spawn_query` prologue.

use super::exec::exec_command_hook;
use super::matching::get_matching_hooks;
use super::parsing::{ParsedHookOutput, parse_hook_output, process_hook_json_output};
use super::{HookBlockingError, HookEvent, HookOutcome, HookResult, RegisteredHooks};
use std::time::Duration;

const TOOL_HOOK_TIMEOUT_MS: u64 = 30_000;

/// Maps to: CC `utils/hooks.ts:1936-1940`
/// `getUserPromptSubmitHookBlockingMessage(...)`.
///
/// Lives here rather than in the `services/hooks` root: the root carries what
/// crosses modules (`HookResult`, `HookEvent`, `create_base_hook_input`), and
/// this is UserPromptSubmit-specific with one consumer. CC keeps it next to
/// `getStopHookMessage` (`:1894`) rather than next to its executor
/// (`:3826`), but that is adjacency inside one 4000-line file, not a grouping
/// this decomposition has to reproduce.
pub fn get_user_prompt_submit_hook_blocking_message(blocking_error: &HookBlockingError) -> String {
    format!(
        "UserPromptSubmit operation blocked by hook:\n{}",
        blocking_error.blocking_error
    )
}

/// Maps to: CC `executeUserPromptSubmitHooks()`.
pub async fn execute_user_prompt_submit_hooks(
    config: &RegisteredHooks,
    prompt: &str,
    permission_mode: &str,
    base_env: Vec<(String, String)>,
) -> Vec<HookResult> {
    // Maps to: CC `hooks.ts:1978-1999`. `executeUserPromptSubmitHooks`
    // (`:3847-3854`) passes no `matchQuery`, so CC's `hookName` is
    // `UserPromptSubmit`.
    if crate::services::hooks::should_skip_hook_execution(HookEvent::UserPromptSubmit, "") {
        return Vec::new();
    }
    let matched = get_matching_hooks(config, HookEvent::UserPromptSubmit, "", None);
    if matched.is_empty() {
        return Vec::new();
    }

    // Maps to: CC `hooks.ts:3841-3845` — `createBaseHookInput(permissionMode)`
    // plus `hook_event_name` and `prompt` only
    // (`UserPromptSubmitHookInputSchema`, `coreSchemas.ts:484-491`).
    // `permission_mode` is a BASE key, which is why it stays; what was missing
    // is `session_id` / `transcript_path` / `cwd`.
    let input_str = super::HookInputBuilder::from_base(super::create_base_hook_input_object(
        &super::HookContext {
            permission_mode: Some(permission_mode.to_string()),
            ..Default::default()
        },
    ))
    .set("hook_event_name", "UserPromptSubmit")
    .set("prompt", prompt)
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

                // CC `hooks.ts:3336-3338`: a hook that exited 0 reports its stdout;
                // one that failed reports its stderr. Reading stdout unconditionally
                // silently discarded every failing hook's message.
                let output = if exec_result.status == 0 {
                    exec_result.stdout.as_str()
                } else {
                    exec_result.stderr.as_str()
                };
                let mut result = match parse_hook_output(output) {
                    ParsedHookOutput::Json(json) => {
                        process_hook_json_output(&json, &command.command)
                    }
                    ParsedHookOutput::PlainText(text) => HookResult {
                        system_message: Some(text),
                        ..Default::default()
                    },
                    _ => HookResult::default(),
                };

                // CC `hooks.ts:3334` `blocked = result.status === 2 || jsonBlocked`,
                // turned into a `blockingError` at `:4396-4402` carrying that same
                // output. Without this the exit-2 convention did nothing here and
                // `apply_user_prompt_submit_hook_results`' blocking branch was dead
                // code — the JSON `decision: 'block'` path is what
                // `process_hook_json_output` already covers.
                let outcome = crate::services::hooks::classify_exit_code(&exec_result);
                if outcome == HookOutcome::Blocking {
                    result.outcome = HookOutcome::Blocking;
                    if result.blocking_error.is_none() {
                        // Trimmed, unlike CC's raw `result.output`: this string is
                        // rendered directly under "blocked by hook:" and above a blank
                        // line, so a trailing newline from `echo` would show as a
                        // second blank line. CC's `execFileNoThrow` boundary is not
                        // ported faithfully enough to know whether it already strips.
                        let message = output.trim();
                        result.blocking_error = Some(HookBlockingError {
                            blocking_error: if message.is_empty() {
                                "Blocked by hook".to_string()
                            } else {
                                message.to_string()
                            },
                            command: command.command.clone(),
                        });
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::hooks::{HookCommand, HookConfigEntry, HooksConfig};

    fn config_with(command: &str) -> RegisteredHooks {
        let mut config = HooksConfig::new();
        config.insert(
            "UserPromptSubmit".to_string(),
            vec![HookConfigEntry {
                matcher: None,
                hooks: vec![HookCommand {
                    command: command.to_string(),
                    shell: None,
                    timeout: Some(5),
                    condition: None,
                    status: None,
                    once: None,
                    is_async: None,
                    async_rewake: None,
                }],
                plugin_root: None,
                plugin_name: None,
                plugin_id: None,
            }],
        );
        crate::services::hooks::test_support::registered_config(&config)
    }

    /// CC `hooks.ts:3334` treats exit 2 as blocked and `:3336-3338` reports a
    /// failed hook's stderr, which `:4396-4402` becomes the blocking message.
    /// Reading stdout unconditionally made the whole exit-2 convention inert.
    #[tokio::test(flavor = "current_thread")]
    async fn exit_code_two_blocks_with_its_stderr_like_official() {
        let _trust = crate::services::hooks::test_support::SessionTrustGuard::accepted();
        let config = config_with("echo 'nope, not this prompt' >&2; exit 2");

        let results =
            execute_user_prompt_submit_hooks(&config, "a prompt", "default", Vec::new()).await;

        assert_eq!(results.len(), 1);
        let blocking = results[0]
            .blocking_error
            .as_ref()
            .expect("exit 2 must produce a blocking error");
        assert_eq!(blocking.blocking_error, "nope, not this prompt");
        assert_eq!(results[0].outcome, HookOutcome::Blocking);
    }

    /// A failing hook with nothing on stderr still blocks; CC falls back to a
    /// fixed string rather than an empty message (`:4399`).
    #[tokio::test(flavor = "current_thread")]
    async fn silent_exit_two_falls_back_to_the_official_default_message() {
        let _trust = crate::services::hooks::test_support::SessionTrustGuard::accepted();
        let config = config_with("exit 2");

        let results =
            execute_user_prompt_submit_hooks(&config, "a prompt", "default", Vec::new()).await;

        assert_eq!(
            results[0]
                .blocking_error
                .as_ref()
                .map(|error| error.blocking_error.as_str()),
            Some("Blocked by hook")
        );
    }

    /// Exit 0 keeps reading stdout, so the success path is unchanged — including
    /// its trailing newline, which the plain-text branch has always preserved.
    /// The blocking branch trims instead; see the note there.
    #[tokio::test(flavor = "current_thread")]
    async fn exit_zero_still_reports_stdout() {
        let _trust = crate::services::hooks::test_support::SessionTrustGuard::accepted();
        let config = config_with("echo 'just a note'");

        let results =
            execute_user_prompt_submit_hooks(&config, "a prompt", "default", Vec::new()).await;

        assert!(results[0].blocking_error.is_none());
        assert_eq!(results[0].system_message.as_deref(), Some("just a note\n"));
    }
}
