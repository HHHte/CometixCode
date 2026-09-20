//! Synthetic structured-output tool for non-interactive/SDK sessions.
//!
//! Maps to: CC `tools/SyntheticOutputTool/SyntheticOutputTool.ts`. The caller
//! supplies the JSON Schema as the active `Tool.input_schema`; the shared tool
//! execution validator enforces it before this behavior returns the captured
//! JSON value.
//!
//! Turn-stop enforcement ("You MUST call the StructuredOutput tool…") is CC's
//! Stop function hook (hookHelpers.ts:70-83, registered at
//! QueryEngine.ts:330-333): the Rust registration lives in
//! utils/hooks/hook_helpers.rs, and until the function-hook engine's Stop
//! evaluation is wired, the query_engine.rs nudge loop is the booked L1
//! stand-in (see the note at its `structured_required` retry block).

/// Maps to CC `SYNTHETIC_OUTPUT_TOOL_NAME`.
pub const SYNTHETIC_OUTPUT_TOOL_NAME: &str = "StructuredOutput";

pub mod ui;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SyntheticOutput {
    pub(crate) data: String,
    pub(crate) structured_output: serde_json::Value,
}

/// Maps to: CC `SyntheticOutputTool.ts:11` — the tool object's zod-facing
/// `inputSchema` is the static `z.object({}).passthrough()`; the caller's
/// JSON Schema (`create_synthetic_output_tool`) governs execution validation
/// separately. The render layer's safeParse gate reads this one.
pub fn input_schema() -> &'static crate::utils::zod::Schema {
    static SCHEMA: std::sync::OnceLock<crate::utils::zod::Schema> = std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| crate::utils::zod::passthrough_object(Vec::new()))
}

/// Maps to CC `isSyntheticOutputToolEnabled`.
pub fn is_synthetic_output_tool_enabled(is_non_interactive_session: bool) -> bool {
    is_non_interactive_session
}

/// Maps to CC `createSyntheticOutputTool(jsonSchema)` after AJV schema
/// validation. Runtime value validation is owned by the shared executor.
///
/// CC memoizes results in a WeakMap keyed by schema object identity
/// (SyntheticOutputTool.ts:105-124, an Ajv-JIT cost optimization for
/// workflow scripts) — Rust values have no object identity, so there is no
/// carrier for that cache and none is invented (pure performance surface).
///
/// The `description` here carries CC's `prompt()` text (:50-52) — the Rust
/// `Tool.description` field is the API tools[].description payload, which CC
/// fills from `prompt()` (confirmed by `create_structured_output_tool`
/// overriding exactly this field ↔ hookHelpers.ts:60-63 overriding
/// `prompt()`). CC's short `description()` string ('Return structured output
/// in the requested format', :47-49) is a UI/telemetry-side label with no
/// Rust carrier yet.
pub fn create_synthetic_output_tool(
    json_schema: serde_json::Value,
) -> Result<crate::types::tools::Tool, String> {
    validate_json_schema_definition(&json_schema)?;
    Ok(crate::types::tools::Tool {
        name: SYNTHETIC_OUTPUT_TOOL_NAME.to_string(),
        description: "Use this tool to return your final response in the requested structured format. You MUST call this tool exactly once at the end of your response to provide the structured output.".to_string(),
        input_schema: json_schema,
        ..Default::default()
    })
}

pub(crate) fn validate_json_schema_definition(schema: &serde_json::Value) -> Result<(), String> {
    // CC uses `new Ajv({ allErrors: true })`, whose default dialect is JSON
    // Schema Draft 7, and surfaces the validator's own text
    // (`ajv.errorsText(ajv.errors)` / the thrown `Error.message`) with no
    // wrapper of its own. Draft 7 also accepts a top-level boolean schema, so
    // the meta-validation pass is what rejects non-schema values. Compiling
    // here (without remote/file retrieval) keeps an accepted schema
    // enforceable locally at tool runtime.
    jsonschema::draft7::meta::validate(schema).map_err(|error| error.to_string())?;
    jsonschema::draft7::new(schema).map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) struct SyntheticOutputTool;

impl crate::tool::ToolCall for SyntheticOutputTool {
    fn name(&self) -> &'static str {
        SYNTHETIC_OUTPUT_TOOL_NAME
    }

    /// Maps to: CC `SyntheticOutputTool.ts:50-52` `async prompt()` — but
    /// INSTANCE-BOUND: `createStructuredOutputTool()` spreads the base tool
    /// and replaces `prompt()` per instance (`utils/hooks/hookHelpers.ts:60-63`).
    /// The port stores each instance's prompt() output in `Tool.description`
    /// (see `create_synthetic_output_tool` doc above), so the instance field is
    /// the faithful read — returning the base literal here would clobber the
    /// hook override.
    fn prompt(
        &self,
        tool: &crate::types::tools::Tool,
        _options: &crate::tool::ToolPromptOptions<'_>,
    ) -> String {
        tool.description.clone()
    }

    fn is_concurrency_safe(&self, _args: &serde_json::Value) -> bool {
        true
    }

    fn is_read_only(&self, _args: &serde_json::Value) -> bool {
        true
    }

    fn search_hint(&self) -> Option<&'static str> {
        Some("return the final response as structured JSON")
    }

    fn max_result_size_chars(&self) -> usize {
        100_000
    }

    fn check_permissions(
        &self,
        args: &serde_json::Value,
        _context: &crate::tool::ToolUseContext,
    ) -> crate::utils::permissions::permission_result::PermissionResult {
        crate::utils::permissions::permission_result::PermissionResult::Allow {
            updated_input: Some(args.clone()),
            user_modified: None,
            decision_reason: None,
            tool_use_id: None,
            accept_feedback: None,
            content_blocks: Vec::new(),
        }
    }

    fn call<'a>(
        &'a self,
        args: &'a serde_json::Value,
        _request: &'a crate::types::permissions::PermissionRequest,
        _context: &'a crate::tool::ToolUseContext,
        _can_use_tool: Option<crate::tool::CanUseToolFn<'a>>,
        _parent_message: Option<&'a crate::types::message::AssistantMessage>,
        _on_progress: Option<crate::tool::ToolCallProgressFn<'a>>,
    ) -> futures::future::BoxFuture<'a, crate::tool::ToolResult> {
        Box::pin(async move {
            crate::tool::ToolResult {
                data: crate::tool::ToolOutput::SyntheticOutput(SyntheticOutput {
                    data: "Structured output provided successfully".to_string(),
                    structured_output: args.clone(),
                }),
                new_messages: Vec::new(),
            }
        })
    }

    fn map_tool_result_to_tool_result_block_param(
        &self,
        data: &crate::tool::ToolOutput,
        _tool_use_id: &str,
    ) -> (String, crate::types::message::ToolResultStatus) {
        match data {
            crate::tool::ToolOutput::SyntheticOutput(output) => (
                output.data.clone(),
                crate::types::message::ToolResultStatus::Success,
            ),
            crate::tool::ToolOutput::Composed {
                content, status, ..
            } => (content.clone(), *status),
            _ => (
                "<tool_use_error>StructuredOutput returned an unexpected output variant</tool_use_error>"
                    .to_string(),
                crate::types::message::ToolResultStatus::Error,
            ),
        }
    }

    /// Maps to: CC recording this tool's `Output` — the bare `data` string
    /// from `call()` (`SyntheticOutputTool.ts:59-65`) — as the message's
    /// `toolUseResult`. The `structured_output` sibling rides the ToolResult
    /// context channel, not the message.
    fn tool_use_result(&self, data: &crate::tool::ToolOutput) -> Option<serde_json::Value> {
        match data {
            crate::tool::ToolOutput::SyntheticOutput(output) => {
                Some(serde_json::Value::String(output.data.clone()))
            }
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
    use crate::tool::ToolCall;

    #[test]
    fn factory_rejects_invalid_schema_and_preserves_valid_dynamic_schema() {
        assert!(
            super::create_synthetic_output_tool(serde_json::json!({
                "type": "object",
                "properties": []
            }))
            .is_err()
        );
        let schema = serde_json::json!({
            "type": "object",
            "properties": {"ok": {"type": "boolean"}},
            "required": ["ok"],
            "additionalProperties": false
        });
        let tool = super::create_synthetic_output_tool(schema.clone()).unwrap();
        assert_eq!(tool.input_schema, schema);
    }

    #[test]
    fn factory_uses_draft7_meta_schema_and_compiles_local_references() {
        for invalid in [
            serde_json::json!({"type": "array", "minItems": "two"}),
            serde_json::json!({"type": "object", "required": ["id", "id"]}),
            serde_json::json!({"type": "object", "properties": {"id": {"$ref": "#/missing"}}}),
        ] {
            assert!(
                super::create_synthetic_output_tool(invalid).is_err(),
                "invalid Draft 7 schema should be rejected"
            );
        }

        let schema = serde_json::json!({
            "type": "object",
            "definitions": {"identifier": {"type": "string", "minLength": 1}},
            "properties": {
                "id": {"$ref": "#/definitions/identifier"},
                "disabled": false
            },
            "required": ["id"],
            "additionalProperties": false
        });
        assert!(super::create_synthetic_output_tool(schema).is_ok());
    }

    #[tokio::test]
    async fn behavior_captures_structured_value() {
        let args = serde_json::json!({"ok": true, "count": 2});
        let request = crate::utils::permissions::permissions::mock_permission_request_with_input(
            "perm-structured".to_string(),
            "toolu-structured".to_string(),
            super::SYNTHETIC_OUTPUT_TOOL_NAME.to_string(),
            args.to_string(),
            args.clone(),
            crate::types::permissions::PermissionMode::Default,
        );
        let result = super::SyntheticOutputTool
            .call(
                &args,
                &request,
                &crate::tool::ToolUseContext::default(),
                None,
                None,
                None,
            )
            .await;
        let crate::tool::ToolOutput::SyntheticOutput(output) = result.data else {
            panic!("expected structured output");
        };
        assert_eq!(output.structured_output, args);
    }
}
