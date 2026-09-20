//! PreToolUse hook execution.
//! Maps to: CC utils/hooks.ts:3394-3436 (executePreToolHooks).
//!
//! Runs matched hooks before a tool executes. Hooks can approve, deny,
//! or modify the tool input. Returns a stream of HookResults.

use super::{HookContext, HookEvent, HookResult, RegisteredHooks};

/// Maps to: CC `utils/hooks.ts:3394-3436#executePreToolHooks`.
pub async fn execute_pre_tool_hooks(
    config: &RegisteredHooks,
    tool_name: &str,
    tool_use_id: &str,
    tool_input: &serde_json::Value,
    hook_context: &HookContext,
    base_env: Vec<(String, String)>,
    abort_controller: Option<&crate::tool::AbortController>,
) -> Vec<HookResult> {
    crate::utils::debug::log_for_debugging(&format!(
        "executePreToolHooks called for tool: {tool_name}"
    ));
    // Maps to: CC `hooks.ts:3418-3424` — the base spread plus exactly four
    // event keys, matching `PreToolUseHookInputSchema`
    // (`entrypoints/sdk/coreSchemas.ts:414-423`). `tool_use_id` belongs to
    // PreToolUse (and Post/Failure/Denied); it is NOT a PermissionRequest key.
    let hook_input = super::tool::tool_event_base_input(hook_context)
        .set("hook_event_name", "PreToolUse")
        .set("tool_name", tool_name)
        .set("tool_input", tool_input.clone())
        .set("tool_use_id", tool_use_id)
        .build();

    super::tool::execute_hooks(
        config,
        HookEvent::PreToolUse,
        tool_name,
        tool_use_id,
        tool_input,
        &hook_input,
        &base_env,
        abort_controller,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::super::HookOutcome;
    use super::super::parsing::process_hook_json_output_for_event;

    #[test]
    fn mismatched_hook_specific_event_cannot_authorize_pre_tool_use() {
        let json: super::super::parsing::HookJsonOutput =
            serde_json::from_value(serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PostToolUse",
                    "permissionDecision": "allow",
                    "updatedInput": {"file_path":"/private/secret"}
                }
            }))
            .unwrap();
        let result = process_hook_json_output_for_event(&json, "bad-hook", "PreToolUse");
        assert_eq!(result.outcome, HookOutcome::NonBlockingError);
        assert!(result.permission_behavior.is_none());
        assert!(result.updated_input.is_none());
    }
}
