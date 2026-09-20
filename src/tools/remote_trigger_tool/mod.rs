//! Incremental port of official `tools/RemoteTriggerTool/*`.
//! Schema/prompt metadata is API-visible only when the hardcoded feature switch
//! enables it. Execution remains safe in `services/tools/tool_execution.rs`: it
//! never calls remote APIs, reads OAuth state, or refreshes tokens.

pub mod prompt;
pub mod ui;

/// Maps to: CC `RemoteTriggerTool` metadata.
/// Maps to: CC `RemoteTriggerTool.ts:18-31` `inputSchema`.
///
/// `body` is `z.record(z.string(), z.unknown())`, which projects the full
/// record shape (`propertyNames` + an open `additionalProperties`) — the
/// hand-written literal it replaces emitted a bare `{"type": "object"}`.
pub fn input_schema() -> &'static crate::utils::zod::Schema {
    static SCHEMA: std::sync::OnceLock<crate::utils::zod::Schema> = std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| {
        use crate::utils::zod;
        zod::strict_object(vec![
            (
                "action",
                zod::enumeration(vec!["list", "get", "create", "update", "run"]),
            ),
            (
                "trigger_id",
                zod::string()
                    .regex(r"^[\w-]+$")
                    .optional()
                    .describe("Required for get, update, and run"),
            ),
            (
                "body",
                zod::record(zod::any())
                    .optional()
                    .describe("JSON body for create and update"),
            ),
        ])
    })
}

pub fn remote_trigger_tool_schema() -> crate::types::tools::Tool {
    crate::types::tools::Tool {
        name: prompt::REMOTE_TRIGGER_TOOL_NAME.to_string(),
        description: prompt::PROMPT.to_string(),
        input_schema: crate::utils::zod_to_json_schema::zod_to_json_schema(input_schema()),
        ..Default::default()
    }
}

/// CC `tools/RemoteTriggerTool/RemoteTriggerTool.ts` `export type Output`
/// (:42) — outputSchema (:35-40). `status` is an unconstrained `z.number()`,
/// so the wire carrier is `serde_json::Number` (same rule as Glob's Number
/// fields).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Output {
    pub(crate) status: serde_json::Number,
    pub(crate) json: String,
}

/// Validate a RemoteTrigger request and return the safe HTTP 501 output.
/// Maps to: CC `tools/RemoteTriggerTool/RemoteTriggerTool.ts` `call` (:78).
/// TODO(parity): official execution refreshes OAuth and calls the remote
/// triggers API. This safe port returns an HTTP 501-shaped output without
/// reading tokens, refreshing OAuth, or sending network requests.
pub(crate) fn remote_trigger_output(input: &serde_json::Value) -> Result<Output, String> {
    let action = input
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("list");
    // CC `if (!trigger_id)` — JS truthiness: only the empty string is falsy
    // (whitespace-only strings pass; the schema regex rejects them upstream).
    let trigger_id = input
        .get("trigger_id")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty());

    let validation_error = match action {
        "list" => None,
        "get" | "run" if trigger_id.is_none() => Some(format!("{action} requires trigger_id")),
        "create" if input.get("body").is_none() => Some("create requires body".to_string()),
        "update" if trigger_id.is_none() => Some("update requires trigger_id".to_string()),
        "update" if input.get("body").is_none() => Some("update requires body".to_string()),
        "get" | "create" | "update" | "run" => None,
        _ => Some(format!("Invalid RemoteTrigger action: {action}")),
    };
    if let Some(error) = validation_error {
        return Err(error);
    }

    let payload = serde_json::json!({
        "error": "Remote trigger execution is unavailable: no OAuth token was read and no remote API request was sent.",
        "action": action,
        "trigger_id": trigger_id,
    });
    Ok(Output {
        status: serde_json::Number::from(501u16),
        json: payload.to_string(),
    })
}

/// Behavioral half of CC `RemoteTriggerTool` — dispatched via `crate::tool::ToolCall`.
pub(crate) struct RemoteTriggerTool;

impl crate::tool::ToolCall for RemoteTriggerTool {
    fn name(&self) -> &'static str {
        "RemoteTrigger"
    }

    /// Maps to: CC `RemoteTriggerTool.ts:75-77` `async prompt() { return
    /// PROMPT }` — same source the wire schema renders eagerly.
    fn prompt(
        &self,
        _tool: &crate::types::tools::Tool,
        _options: &crate::tool::ToolPromptOptions<'_>,
    ) -> String {
        prompt::PROMPT.to_string()
    }

    /// Maps to: CC `RemoteTriggerTool.ts:57-62` `isEnabled()` — the runtime
    /// registry consults this trait method; `get_tools` filters the API
    /// payload through the same predicate.
    fn is_enabled(&self) -> bool {
        prompt::is_remote_trigger_tool_enabled()
    }

    /// Maps to: CC `RemoteTriggerTool.isConcurrencySafe()` (:63-65).
    fn is_concurrency_safe(&self, _args: &serde_json::Value) -> bool {
        true
    }

    /// Maps to: CC `RemoteTriggerTool.isReadOnly(input)` (:66-68).
    fn is_read_only(&self, args: &serde_json::Value) -> bool {
        matches!(
            args.get("action").and_then(serde_json::Value::as_str),
            Some("list" | "get")
        )
    }

    /// Maps to: CC `RemoteTriggerTool.searchHint` (:48).
    fn search_hint(&self) -> Option<&'static str> {
        Some("manage scheduled remote agent triggers")
    }

    /// Maps to: CC `RemoteTriggerTool.ts:72-74` `description()`.
    fn description(&self, _args: &serde_json::Value) -> String {
        prompt::DESCRIPTION.to_string()
    }

    /// Maps to: CC `RemoteTriggerTool.maxResultSizeChars` (:49).
    fn max_result_size_chars(&self) -> usize {
        100_000
    }

    /// Maps to: CC `RemoteTriggerTool.shouldDefer` (:50).
    fn should_defer(&self) -> bool {
        true
    }

    /// Maps to: CC `RemoteTriggerTool.toAutoClassifierInput(input)` (:69-71).
    fn to_auto_classifier_input(&self, args: &serde_json::Value) -> String {
        let action = args
            .get("action")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        match args.get("trigger_id").and_then(serde_json::Value::as_str) {
            Some(trigger_id) if !trigger_id.is_empty() => {
                format!("RemoteTrigger {action} {trigger_id}")
            }
            _ => format!("RemoteTrigger {action}"),
        }
    }

    fn call<'a>(
        &'a self,
        args: &'a serde_json::Value,
        request: &'a crate::types::permissions::PermissionRequest,
        _context: &'a crate::tool::ToolUseContext,
        _can_use_tool: Option<crate::tool::CanUseToolFn<'a>>,
        _parent_message: Option<&'a crate::types::message::AssistantMessage>,
        _on_progress: Option<crate::tool::ToolCallProgressFn<'a>>,
    ) -> futures::future::BoxFuture<'a, crate::tool::ToolResult> {
        Box::pin(async move {
            let _ = request;
            match remote_trigger_output(args) {
                Ok(output) => crate::tool::ToolResult {
                    data: crate::tool::ToolOutput::RemoteTrigger(output),
                    new_messages: Vec::new(),
                },
                Err(error) => crate::tool::ToolResult {
                    data: crate::tool::ToolOutput::Composed {
                        content: error,
                        status: crate::types::message::ToolResultStatus::Error,
                    },
                    new_messages: Vec::new(),
                },
            }
        })
    }

    /// Maps to: CC `tools/RemoteTriggerTool/RemoteTriggerTool.ts`
    /// `mapToolResultToToolResultBlockParam` (:152-158).
    fn map_tool_result_to_tool_result_block_param(
        &self,
        data: &crate::tool::ToolOutput,
        _tool_use_id: &str,
    ) -> (String, crate::types::message::ToolResultStatus) {
        match data {
            // CC pairs `validateStatus: () => true` with an `is_error`-free
            // result so the model reads the raw status code itself.
            crate::tool::ToolOutput::RemoteTrigger(output) => (
                format!("HTTP {}\n{}", output.status, output.json),
                crate::types::message::ToolResultStatus::Success,
            ),
            crate::tool::ToolOutput::Composed {
                content, status, ..
            } => (content.clone(), *status),
            _ => (
                "<tool_use_error>RemoteTrigger returned an unexpected output variant</tool_use_error>"
                    .to_string(),
                crate::types::message::ToolResultStatus::Error,
            ),
        }
    }

    /// Maps to: CC recording RemoteTriggerTool's `Output` as the message's
    /// `toolUseResult`.
    fn tool_use_result(&self, data: &crate::tool::ToolOutput) -> Option<serde_json::Value> {
        match data {
            crate::tool::ToolOutput::RemoteTrigger(output) => Some(ui::output_to_value(output)),
            crate::tool::ToolOutput::Composed {
                content,
                status: crate::types::message::ToolResultStatus::Error,
                ..
            } => {
                let message = crate::utils::messages::extract_tag(content, "tool_use_error")
                    .unwrap_or_else(|| content.clone());
                Some(serde_json::Value::String(message))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn remote_trigger_tool_schema_matches_official_input_shape() {
        let schema = super::remote_trigger_tool_schema();
        assert_eq!(schema.name, "RemoteTrigger");
        assert_eq!(
            schema.input_schema.get("required"),
            Some(&serde_json::json!(["action"]))
        );
        assert!(
            schema
                .input_schema
                .pointer("/properties/action/enum")
                .and_then(|value| value.as_array())
                .is_some_and(|values| values.iter().any(|value| value == "run"))
        );
        assert!(schema.description.contains("remote-trigger API"));
    }

    #[tokio::test]
    async fn remote_trigger_tool_call_returns_official_output_schema_and_model_copy() {
        use crate::tool::ToolCall;

        let args = serde_json::json!({"action": "run", "trigger_id": "daily-check"});
        let request = crate::utils::permissions::permissions::mock_permission_request_with_input(
            "perm-remote-trigger".to_string(),
            "toolu_remote_trigger".to_string(),
            "RemoteTrigger".to_string(),
            "run daily-check".to_string(),
            args.clone(),
            crate::types::permissions::PermissionMode::Default,
        );
        let tool = super::RemoteTriggerTool;
        let result = tool
            .call(
                &args,
                &request,
                &crate::tool::ToolUseContext::default(),
                None,
                None,
                None,
            )
            .await;

        let crate::tool::ToolOutput::RemoteTrigger(output) = result.data else {
            panic!("RemoteTrigger should return its official ToolOutput variant");
        };
        assert_eq!(output.status.as_u64(), Some(501));
        assert!(output.json.contains("daily-check"));
        assert!(output.json.contains("no OAuth token was read"));
        assert!(!tool.is_read_only(&args));
        assert!(tool.is_read_only(&serde_json::json!({"action": "list"})));
        assert_eq!(
            tool.to_auto_classifier_input(&args),
            "RemoteTrigger run daily-check"
        );
        assert_eq!(
            tool.to_auto_classifier_input(&serde_json::json!({"action": "list"})),
            "RemoteTrigger list"
        );

        let data = crate::tool::ToolOutput::RemoteTrigger(output);
        let (content, status) =
            tool.map_tool_result_to_tool_result_block_param(&data, "toolu_remote_trigger");
        // CC never sets is_error here; the status code is the model's signal.
        assert_eq!(status, crate::types::message::ToolResultStatus::Success);
        assert!(content.starts_with("HTTP 501\n"));
        assert!(content.contains("daily-check"));
        // No display shape — the trait projects the raw Output object.
        let raw = tool
            .tool_use_result(&data)
            .expect("raw output should ride the row");
        assert_eq!(raw.get("status"), Some(&serde_json::json!(501)));
        assert!(
            raw.get("json")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|json| json.contains("daily-check"))
        );
    }
}
