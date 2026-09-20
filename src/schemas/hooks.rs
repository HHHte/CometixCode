//! Hook Zod schemas extracted to break import cycles.
//!
//! Maps to: CC `schemas/hooks.ts` — hook-related schema definitions that were
//! originally in `utils/settings/types.ts`, extracted so settings/types and
//! plugins/schemas both import from this shared location instead of each
//! other.
//!
//! The serde types below are the post-parse projections of the schemas' `z.infer`
//! exports (the `FileReadInput` pattern); the `*_schema()` functions are CC's
//! `lazySchema` factories in their settled Rust shape (`static OnceLock`).

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::services::hooks::HOOK_EVENTS;
use crate::utils::shell::shell_provider::SHELL_TYPES;
use crate::utils::zod::{self, Schema};

// ════════════════════════════════════════════════════════════
// Inferred types — map to CC `z.infer` exports at schemas/hooks.ts:216-222
// ════════════════════════════════════════════════════════════

/// A user-configured shell command hook.
///
/// Maps to: CC `HookCommand` (`schemas/hooks.ts:217`). The serde shape covers
/// the `command` arm's fields — the arm the execution chain consumes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookCommand {
    /// Shell command string to execute.
    pub command: String,
    /// Optional shell type override (default: system shell).
    pub shell: Option<String>,
    /// Optional timeout in seconds.
    pub timeout: Option<u64>,
    /// Conditional execution pattern (tool input matcher).
    /// Maps to: CC schemas/hooks.ts `if` field.
    #[serde(rename = "if")]
    pub condition: Option<String>,
    /// Display message shown during hook execution.
    /// Maps to: CC schemas/hooks.ts `statusMessage` field.
    #[serde(rename = "statusMessage", alias = "status")]
    pub status: Option<String>,
    /// Run-once flag — hook is removed after first execution.
    /// Maps to: CC schemas/hooks.ts `once` field.
    pub once: Option<bool>,
    /// Background execution flag — hook runs asynchronously.
    /// Maps to: CC schemas/hooks.ts `async` field.
    #[serde(rename = "async")]
    pub is_async: Option<bool>,
    /// Rewake model on exit code 2 for background hooks.
    /// Maps to: CC schemas/hooks.ts `asyncRewake` field.
    pub async_rewake: Option<bool>,
}

/// Maps to: CC `schemas/hooks.ts#AgentHook` (agentHookSchema inferred type).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentHook {
    pub prompt: String,
    pub timeout: Option<f64>,
    pub model: Option<String>,
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub status_message: Option<String>,
    pub once: Option<bool>,
}

/// A single hook configuration entry from settings.
///
/// Maps to: CC `HookMatcher` (`schemas/hooks.ts:221`), widened with the
/// `PluginHookMatcher` bookkeeping fields (`utils/settings/types.ts`) the
/// loading chain attaches — CC keeps those on a separate extending type; the
/// merged shape here is the settled Rust carrier for both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookConfigEntry {
    /// Pattern to match against (tool name, event name, etc.).
    /// Empty or "*" matches everything.
    pub matcher: Option<String>,
    /// Hook commands to execute when matched.
    pub hooks: Vec<HookCommand>,
    /// Maps to: CC `PluginHookMatcher.pluginRoot`.
    #[serde(skip)]
    pub plugin_root: Option<String>,
    /// Maps to: CC `PluginHookMatcher.pluginName`.
    #[serde(skip)]
    pub plugin_name: Option<String>,
    /// Maps to: CC `PluginHookMatcher.pluginId`.
    #[serde(skip)]
    pub plugin_id: Option<String>,
}

/// Hooks configuration grouped by event type.
///
/// Maps to: CC `HooksSettings` (`schemas/hooks.ts:222`) —
/// `Partial<Record<HookEvent, HookMatcher[]>>`.
pub type HooksConfig = std::collections::HashMap<String, Vec<HookConfigEntry>>;

/// Maps to: CC `HookCallback` (`types/hooks.ts:211-226`) — an SDK-registered
/// hook resolved by calling back into the consumer instead of spawning a
/// command. The Rust callback receives (input, tool_use_id) and returns the
/// raw HookJSONOutput value; the print leg validates it against
/// `hook_json_output_schema` before it reaches the execution chain (CC's
/// `sendRequest(­hookJSONOutputSchema())` does the same). The abort signal,
/// hookIndex, and app-state context legs of CC's signature have no Rust
/// counterpart on this seam yet.
#[derive(Clone)]
pub struct HookCallback {
    pub callback: std::sync::Arc<
        dyn Fn(
                serde_json::Value,
                Option<String>,
            ) -> futures::future::BoxFuture<'static, serde_json::Value>
            + Send
            + Sync,
    >,
    /// Timeout in seconds for this hook.
    pub timeout: Option<u64>,
}

impl std::fmt::Debug for HookCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookCallback")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl PartialEq for HookCallback {
    fn eq(&self, other: &Self) -> bool {
        self.timeout == other.timeout
    }
}

/// Maps to: CC `utils/hooks.ts:356` — the union the execution chain consumes
/// (`HookCommand | HookCallback`; the `FunctionHook` arm is session-storage
/// internal and has no Rust counterpart yet).
#[derive(Clone, Debug, PartialEq)]
pub enum RegisteredHook {
    Command(HookCommand),
    Callback(HookCallback),
}

impl RegisteredHook {
    /// The hook's own timeout (seconds), whichever arm carries it.
    pub fn timeout(&self) -> Option<u64> {
        match self {
            RegisteredHook::Command(command) => command.timeout,
            RegisteredHook::Callback(callback) => callback.timeout,
        }
    }
}

/// Maps to: CC `HookCallbackMatcher` (`types/hooks.ts:228-232`) merged with
/// the `PluginHookMatcher` bookkeeping the loading chain attaches — the
/// matcher shape the REGISTERED (SDK/plugin) channel and the execution chain
/// share once settings entries are folded in.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RegisteredHookMatcher {
    pub matcher: Option<String>,
    pub hooks: Vec<RegisteredHook>,
    pub plugin_root: Option<String>,
    pub plugin_name: Option<String>,
    pub plugin_id: Option<String>,
}

impl RegisteredHookMatcher {
    /// Fold a settings-sourced entry into the execution-chain shape (the
    /// merge point CC's `getHooksConfig` reaches by typing).
    pub fn from_config_entry(entry: &HookConfigEntry) -> Self {
        RegisteredHookMatcher {
            matcher: entry.matcher.clone(),
            hooks: entry
                .hooks
                .iter()
                .cloned()
                .map(RegisteredHook::Command)
                .collect(),
            plugin_root: entry.plugin_root.clone(),
            plugin_name: entry.plugin_name.clone(),
            plugin_id: entry.plugin_id.clone(),
        }
    }
}

/// The merged, execution-facing hooks table (settings + registered sources).
pub type RegisteredHooks = std::collections::HashMap<String, Vec<RegisteredHookMatcher>>;

// ════════════════════════════════════════════════════════════
// Zod carrier schemas
// ════════════════════════════════════════════════════════════

/// Maps to: CC `IfConditionSchema` (`schemas/hooks.ts:19-27`) — shared `if`
/// condition field using permission rule syntax to filter hooks before
/// spawning. (CC memoizes via lazySchema; the factory here is pure, so the
/// build sites clone an identical value.)
fn if_condition_schema() -> Schema {
    zod::string().optional().describe(
        "Permission rule syntax to filter when this hook runs (e.g., \"Bash(git *)\"). Only runs if the tool call matches the pattern. Avoids spawning hooks for non-matching commands.",
    )
}

/// Maps to: CC `buildHookSchemas()` (`schemas/hooks.ts:31-173`) — internal
/// factory for the four hook variants. Returns (command, prompt, http, agent)
/// in CC's declaration order.
fn build_hook_schemas() -> (Schema, Schema, Schema, Schema) {
    let bash_command_hook_schema = zod::object(vec![
        (
            "type",
            zod::literal(serde_json::json!("command")).describe("Shell command hook type"),
        ),
        ("command", zod::string().describe("Shell command to execute")),
        ("if", if_condition_schema()),
        (
            "shell",
            zod::enumeration(SHELL_TYPES.iter().map(|s| s.as_str()).collect())
                .optional()
                .describe(
                    "Shell interpreter. 'bash' uses your $SHELL (bash/zsh/sh); 'powershell' uses pwsh. Defaults to bash.",
                ),
        ),
        (
            "timeout",
            zod::number()
                .positive()
                .optional()
                .describe("Timeout in seconds for this specific command"),
        ),
        (
            "statusMessage",
            zod::string()
                .optional()
                .describe("Custom status message to display in spinner while hook runs"),
        ),
        (
            "once",
            zod::boolean()
                .optional()
                .describe("If true, hook runs once and is removed after execution"),
        ),
        (
            "async",
            zod::boolean()
                .optional()
                .describe("If true, hook runs in background without blocking"),
        ),
        (
            "asyncRewake",
            zod::boolean().optional().describe(
                "If true, hook runs in background and wakes the model on exit code 2 (blocking error). Implies async.",
            ),
        ),
    ]);

    let prompt_hook_schema = zod::object(vec![
        (
            "type",
            zod::literal(serde_json::json!("prompt")).describe("LLM prompt hook type"),
        ),
        (
            "prompt",
            zod::string().describe(
                "Prompt to evaluate with LLM. Use $ARGUMENTS placeholder for hook input JSON.",
            ),
        ),
        ("if", if_condition_schema()),
        (
            "timeout",
            zod::number()
                .positive()
                .optional()
                .describe("Timeout in seconds for this specific prompt evaluation"),
        ),
        (
            "model",
            zod::string().optional().describe(
                "Model to use for this prompt hook (e.g., \"claude-sonnet-4-6\"). If not specified, uses the default small fast model.",
            ),
        ),
        (
            "statusMessage",
            zod::string()
                .optional()
                .describe("Custom status message to display in spinner while hook runs"),
        ),
        (
            "once",
            zod::boolean()
                .optional()
                .describe("If true, hook runs once and is removed after execution"),
        ),
    ]);

    let http_hook_schema = zod::object(vec![
        (
            "type",
            zod::literal(serde_json::json!("http")).describe("HTTP hook type"),
        ),
        (
            "url",
            zod::string()
                .url()
                .describe("URL to POST the hook input JSON to"),
        ),
        ("if", if_condition_schema()),
        (
            "timeout",
            zod::number()
                .positive()
                .optional()
                .describe("Timeout in seconds for this specific request"),
        ),
        (
            "headers",
            zod::record(zod::string()).optional().describe(
                "Additional headers to include in the request. Values may reference environment variables using $VAR_NAME or ${VAR_NAME} syntax (e.g., \"Authorization\": \"Bearer $MY_TOKEN\"). Only variables listed in allowedEnvVars will be interpolated.",
            ),
        ),
        (
            "allowedEnvVars",
            zod::array(zod::string()).optional().describe(
                "Explicit list of environment variable names that may be interpolated in header values. Only variables listed here will be resolved; all other $VAR references are left as empty strings. Required for env var interpolation to work.",
            ),
        ),
        (
            "statusMessage",
            zod::string()
                .optional()
                .describe("Custom status message to display in spinner while hook runs"),
        ),
        (
            "once",
            zod::boolean()
                .optional()
                .describe("If true, hook runs once and is removed after execution"),
        ),
    ]);

    // No transform on the agent prompt: this schema feeds parseSettingsFile,
    // and updateSettingsForSource round-trips the parsed result through
    // JSON.stringify — a transformed function value would be silently dropped,
    // deleting the user's prompt from settings.json (gh-24920, CC-79).
    let agent_hook_schema = zod::object(vec![
        (
            "type",
            zod::literal(serde_json::json!("agent")).describe("Agentic verifier hook type"),
        ),
        (
            "prompt",
            zod::string().describe(
                "Prompt describing what to verify (e.g. \"Verify that unit tests ran and passed.\"). Use $ARGUMENTS placeholder for hook input JSON.",
            ),
        ),
        ("if", if_condition_schema()),
        (
            "timeout",
            zod::number()
                .positive()
                .optional()
                .describe("Timeout in seconds for agent execution (default 60)"),
        ),
        (
            "model",
            zod::string().optional().describe(
                "Model to use for this agent hook (e.g., \"claude-sonnet-4-6\"). If not specified, uses Haiku.",
            ),
        ),
        (
            "statusMessage",
            zod::string()
                .optional()
                .describe("Custom status message to display in spinner while hook runs"),
        ),
        (
            "once",
            zod::boolean()
                .optional()
                .describe("If true, hook runs once and is removed after execution"),
        ),
    ]);

    (
        bash_command_hook_schema,
        prompt_hook_schema,
        http_hook_schema,
        agent_hook_schema,
    )
}

/// Maps to: CC `HookCommandSchema` (`schemas/hooks.ts:176-190`) — hook command
/// schema (excludes function hooks — they can't be persisted). Union order is
/// CC's: command, prompt, agent, http.
pub fn hook_command_schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let (bash_command, prompt, http, agent) = build_hook_schemas();
        zod::discriminated_union("type", vec![bash_command, prompt, agent, http])
    })
}

/// Maps to: CC `HookMatcherSchema` (`schemas/hooks.ts:194-205`) — matcher
/// configuration with multiple hooks.
pub fn hook_matcher_schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        zod::object(vec![
            (
                "matcher",
                zod::string()
                    .optional()
                    .describe("String pattern to match (e.g. tool names like \"Write\")"),
            ),
            (
                "hooks",
                zod::array(hook_command_schema().clone())
                    .describe("List of hooks to execute when the matcher matches"),
            ),
        ])
    })
}

/// Maps to: CC `HooksSchema` (`schemas/hooks.ts:211-213`) — hooks
/// configuration keyed by hook event; partialRecord since not all hook events
/// need to be defined.
pub fn hooks_schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        zod::partial_record(
            HOOK_EVENTS.iter().map(|e| e.as_str()).collect(),
            zod::array(hook_matcher_schema().clone()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::zod::{safe_parse, to_json_schema};
    use serde_json::json;

    #[test]
    fn hooks_schema_accepts_partial_events_and_rejects_unknown_keys() {
        let ok = json!({
            "PreToolUse": [{
                "matcher": "Bash",
                "hooks": [{"type": "command", "command": "echo hi", "timeout": 5}],
            }],
            "SessionStart": [{
                "hooks": [{"type": "prompt", "prompt": "check $ARGUMENTS"}],
            }],
        });
        assert!(safe_parse(hooks_schema(), &ok).is_ok());

        // An unknown event key is invalid_key with the key on the path.
        let error = safe_parse(hooks_schema(), &json!({"NotAnEvent": []}))
            .expect_err("unknown event fails");
        assert_eq!(error.issues[0].code.as_str(), "invalid_key");
        assert_eq!(error.issues[0].message, "Invalid key in record");

        // A hook missing its discriminator arm fields fails inside the arm.
        let bad_hook = json!({
            "PreToolUse": [{"hooks": [{"type": "command"}]}],
        });
        assert!(safe_parse(hooks_schema(), &bad_hook).is_err());
    }

    #[test]
    fn hook_command_union_discriminates_all_four_arms() {
        let s = hook_command_schema();
        assert!(safe_parse(s, &json!({"type": "command", "command": "ls"})).is_ok());
        assert!(safe_parse(s, &json!({"type": "prompt", "prompt": "p"})).is_ok());
        assert!(safe_parse(s, &json!({"type": "agent", "prompt": "verify"})).is_ok());
        assert!(safe_parse(s, &json!({"type": "http", "url": "https://x.test/hook"})).is_ok());
        // http requires a URL-shaped url.
        assert!(safe_parse(s, &json!({"type": "http", "url": "not a url"})).is_err());
        // Unknown discriminator fails on the tag path.
        let error =
            safe_parse(s, &json!({"type": "function"})).expect_err("function hooks not persisted");
        assert_eq!(error.issues[0].path.len(), 1);
    }

    #[test]
    fn hooks_projection_names_every_event_key() {
        let projected = to_json_schema(hooks_schema());
        let keys = projected["propertyNames"]["anyOf"][0]["enum"]
            .as_array()
            .expect("enum keys");
        assert_eq!(keys.len(), HOOK_EVENTS.len());
        assert!(keys.contains(&json!("PreToolUse")));
        // The value side is the matcher array with the union of four arms.
        assert_eq!(
            projected["additionalProperties"]["items"]["properties"]["hooks"]["items"]["anyOf"]
                .as_array()
                .map(Vec::len),
            Some(4)
        );
    }
}
