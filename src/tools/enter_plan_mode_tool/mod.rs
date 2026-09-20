//! EnterPlanMode tool metadata, UI, and call path.
//!
//! Maps to:
//! - CC `tools/EnterPlanModeTool/EnterPlanModeTool.ts`
//! - CC `tools/EnterPlanModeTool/constants.ts`
//! - CC `tools/EnterPlanModeTool/prompt.ts`
//! - CC `tools/EnterPlanModeTool/UI.tsx`

pub mod constants;
pub mod prompt;
pub mod ui;

/// Maps to: CC `EnterPlanModeTool.ts:21-25` `inputSchema` — `z.strictObject({})`,
/// no parameters. Built once per process, as CC's `lazySchema()` does.
pub fn input_schema() -> &'static crate::utils::zod::Schema {
    static SCHEMA: std::sync::OnceLock<crate::utils::zod::Schema> = std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| crate::utils::zod::strict_object(vec![]))
}

pub fn enter_plan_mode_tool_schema() -> crate::types::tools::Tool {
    crate::types::tools::Tool {
        name: constants::ENTER_PLAN_MODE_TOOL_NAME.to_string(),
        description: prompt::get_enter_plan_mode_tool_prompt(),
        input_schema: crate::utils::zod_to_json_schema::zod_to_json_schema(input_schema()),
        ..Default::default()
    }
}

/// CC `tools/EnterPlanModeTool/EnterPlanModeTool.ts` outputSchema (:28-32).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Output {
    pub(crate) message: String,
}

/// Official EnterPlanMode acknowledgement output.
pub(crate) fn enter_plan_mode_output() -> Output {
    Output {
        message: "Entered plan mode. You should now focus on exploring the codebase and designing an implementation approach."
            .to_string(),
    }
}

fn enter_plan_mode_model_content(message: &str) -> String {
    // Maps to CC `mapToolResultToToolResultBlockParam` (:103-118). The same
    // `isPlanModeInterviewPhaseEnabled()` gate drives the prompt's
    // WHAT_HAPPENS_SECTION, so both must read it.
    if crate::utils::plan_mode_v2::is_plan_mode_interview_phase_enabled() {
        return format!(
            "{message}\n\nDO NOT write or edit any files except the plan file. Detailed workflow instructions will follow."
        );
    }
    format!(
        "{message}\n\nIn plan mode, you should:\n1. Thoroughly explore the codebase to understand existing patterns\n2. Identify similar features and architectural approaches\n3. Consider multiple approaches and their trade-offs\n4. Use AskUserQuestion if you need to clarify the approach\n5. Design a concrete implementation strategy\n6. When ready, use ExitPlanMode to present your plan for approval\n\nRemember: DO NOT write or edit any files yet. This is a read-only exploration and planning phase."
    )
}

/// Maps to: CC `EnterPlanModeTool.call` (:77-101).
pub(crate) fn enter_plan_mode_call(
    context: &crate::tool::ToolUseContext,
) -> Result<Output, String> {
    use crate::types::permissions::{
        PermissionMode, PermissionUpdate, PermissionUpdateDestination,
    };
    use crate::utils::permissions::permission_mode::to_external_permission_mode;
    use crate::utils::permissions::permission_setup::prepare_context_for_plan_mode;
    use crate::utils::permissions::permission_update::apply_permission_update;

    if context.agent_id.is_some() {
        return Err("EnterPlanMode tool cannot be used in agent contexts".to_string());
    }

    // Prefer live AppState; fall back to ToolUseContext.tool_permission_context.
    let current_mode = context
        .get_app_state()
        .map(|state| state.tool_permission_context.mode)
        .unwrap_or(context.tool_permission_context.mode);
    let from_mode = to_external_permission_mode(current_mode);
    crate::bootstrap::state::handle_plan_mode_transition(from_mode, "plan");

    if let Some(_store) = context.app_store.store.as_ref() {
        context.set_app_state(|state| {
            let prepared = prepare_context_for_plan_mode(&state.tool_permission_context);
            let updated = apply_permission_update(
                &prepared,
                &PermissionUpdate::SetMode {
                    mode: PermissionMode::Plan,
                    destination: PermissionUpdateDestination::Session,
                },
            );
            state.set_tool_permission_context(updated);
        });
    } else {
        // No live store (unit tests / headless call) — mutate context copy only.
        // ToolUseContext is not interior-mutated here; permission mode change
        // requires AppStore for session effect. Call still returns success
        // message so model copy matches official.
        tracing::debug!("[EnterPlanMode] no AppStore; mode transition skipped");
    }

    Ok(enter_plan_mode_output())
}

/// Behavioral half of CC `EnterPlanModeTool` — dispatched via `crate::tool::ToolCall`.
pub(crate) struct EnterPlanModeTool;

impl crate::tool::ToolCall for EnterPlanModeTool {
    fn name(&self) -> &'static str {
        "EnterPlanMode"
    }

    /// Maps to: CC `EnterPlanModeTool.ts:43-45` `async prompt() { return
    /// getEnterPlanModeToolPrompt() }` — same source the wire schema renders.
    fn prompt(
        &self,
        _tool: &crate::types::tools::Tool,
        _options: &crate::tool::ToolPromptOptions<'_>,
    ) -> String {
        prompt::get_enter_plan_mode_tool_prompt()
    }

    fn search_hint(&self) -> Option<&'static str> {
        Some("switch to plan mode to design an approach before coding")
    }

    /// Maps to: CC `EnterPlanModeTool.ts:39` `maxResultSizeChars`.
    fn max_result_size_chars(&self) -> usize {
        100_000
    }

    /// Maps to: CC `EnterPlanModeTool.ts:40-42` `description()`.
    fn description(&self, _args: &serde_json::Value) -> String {
        "Requests permission to enter plan mode for complex tasks requiring exploration and design"
            .to_string()
    }

    /// Maps to: CC `EnterPlanModeTool.ts:52-54` `userFacingName()` — the
    /// empty string.
    fn user_facing_name(&self, _args: Option<&serde_json::Value>) -> String {
        String::new()
    }

    fn should_defer(&self) -> bool {
        true
    }

    /// Maps to: CC `EnterPlanModeTool.isEnabled()` channels guard (:56-67):
    /// `(feature('KAIROS') || feature('KAIROS_CHANNELS')) &&
    /// getAllowedChannels().length > 0` → false. These are build features,
    /// not the `tengu_harbor` GrowthBook gate. Paired with the same gate on
    /// ExitPlanMode so plan mode isn't a trap.
    fn is_enabled(&self) -> bool {
        let kairos_built = crate::utils::feature_flags::feature_enabled(
            crate::utils::feature_flags::FeatureFlag::Kairos,
        ) || crate::utils::feature_flags::feature_enabled(
            crate::utils::feature_flags::FeatureFlag::KairosChannels,
        );
        if kairos_built && !crate::bootstrap::state::get_allowed_channels().is_empty() {
            return false;
        }
        true
    }

    fn is_concurrency_safe(&self, _args: &serde_json::Value) -> bool {
        true
    }

    fn is_read_only(&self, _args: &serde_json::Value) -> bool {
        true
    }

    fn call<'a>(
        &'a self,
        args: &'a serde_json::Value,
        request: &'a crate::types::permissions::PermissionRequest,
        context: &'a crate::tool::ToolUseContext,
        _can_use_tool: Option<crate::tool::CanUseToolFn<'a>>,
        _parent_message: Option<&'a crate::types::message::AssistantMessage>,
        _on_progress: Option<crate::tool::ToolCallProgressFn<'a>>,
    ) -> futures::future::BoxFuture<'a, crate::tool::ToolResult> {
        Box::pin(async move {
            let _ = args;
            let _ = request;
            match enter_plan_mode_call(context) {
                Ok(output) => crate::tool::ToolResult {
                    data: crate::tool::ToolOutput::EnterPlanMode(output),
                    new_messages: Vec::new(),
                },
                Err(message) => crate::tool::ToolResult {
                    data: crate::tool::ToolOutput::Composed {
                        content: format!("Error: {message}"),
                        status: crate::types::message::ToolResultStatus::Error,
                    },
                    new_messages: Vec::new(),
                },
            }
        })
    }

    fn map_tool_result_to_tool_result_block_param(
        &self,
        data: &crate::tool::ToolOutput,
        _tool_use_id: &str,
    ) -> (String, crate::types::message::ToolResultStatus) {
        match data {
            crate::tool::ToolOutput::EnterPlanMode(output) => (
                enter_plan_mode_model_content(&output.message),
                crate::types::message::ToolResultStatus::Success,
            ),
            crate::tool::ToolOutput::Composed {
                content, status, ..
            } => (content.clone(), *status),
            _ => (
                "<tool_use_error>EnterPlanMode returned an unexpected output variant</tool_use_error>"
                    .to_string(),
                crate::types::message::ToolResultStatus::Error,
            ),
        }
    }

    /// Maps to: CC recording EnterPlanModeTool's `Output` as the message's
    /// `toolUseResult`.
    fn tool_use_result(&self, data: &crate::tool::ToolOutput) -> Option<serde_json::Value> {
        match data {
            crate::tool::ToolOutput::EnterPlanMode(output) => Some(ui::output_to_value(output)),
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
    use super::*;

    #[test]
    fn enter_plan_mode_tool_schema_matches_official_empty_input_shape() {
        let schema = enter_plan_mode_tool_schema();
        assert_eq!(schema.name, "EnterPlanMode");
        // zod omits `required` when nothing is required; the empty array this
        // used to assert was the hand-written helper's invention.
        assert_eq!(schema.input_schema.get("required"), None);
        assert!(
            schema
                .input_schema
                .get("properties")
                .and_then(|value| value.as_object())
                .is_some_and(|properties| properties.is_empty())
        );
        assert!(schema.description.contains("When to Use This Tool"));
        // WHAT_HAPPENS_SECTION is present exactly when the interview phase is
        // off, which is the default for external builds.
        assert_eq!(
            schema.description.contains("## What Happens in Plan Mode"),
            !crate::utils::plan_mode_v2::is_plan_mode_interview_phase_enabled()
        );
    }

    #[tokio::test]
    async fn enter_plan_mode_tool_call_returns_official_output_schema_and_model_copy() {
        use crate::tool::ToolCall;

        let args = serde_json::json!({});
        let request = crate::utils::permissions::permissions::mock_permission_request_with_input(
            "perm-enter-plan".to_string(),
            "toolu_enter_plan".to_string(),
            "EnterPlanMode".to_string(),
            "{}".to_string(),
            args.clone(),
            crate::types::permissions::PermissionMode::Default,
        );
        let tool = EnterPlanModeTool;
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

        let crate::tool::ToolOutput::EnterPlanMode(output) = result.data else {
            panic!("EnterPlanMode should return its official ToolOutput variant");
        };
        assert!(output.message.starts_with("Entered plan mode."));

        let data = crate::tool::ToolOutput::EnterPlanMode(output);
        let (content, status) =
            tool.map_tool_result_to_tool_result_block_param(&data, "toolu_enter_plan");
        assert_eq!(status, crate::types::message::ToolResultStatus::Success);
        if crate::utils::plan_mode_v2::is_plan_mode_interview_phase_enabled() {
            assert!(content.contains(
                "DO NOT write or edit any files except the plan file. Detailed workflow instructions will follow."
            ));
        } else {
            assert!(content.contains("In plan mode, you should:"));
            assert!(content.contains("Remember: DO NOT write or edit any files yet"));
        }
        // No display shape — the trait projects the raw Output object.
        let raw = tool
            .tool_use_result(&data)
            .expect("raw output should ride the row");
        assert!(
            raw.get("message")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|message| message.starts_with("Entered plan mode."))
        );

        // Explicit CC member values (EnterPlanModeTool.ts:39-42, :52-54).
        assert_eq!(tool.max_result_size_chars(), 100_000);
        assert_eq!(tool.user_facing_name(None), "");
        assert_eq!(
            tool.description(&serde_json::json!({})),
            "Requests permission to enter plan mode for complex tasks requiring exploration and design"
        );
    }

    #[test]
    fn enter_plan_mode_with_app_store_sets_plan_mode_and_pre_plan() {
        use crate::state::app_state_store::AppState;
        use crate::state::store::AppStore;
        use crate::types::permissions::PermissionMode;

        let mut initial = AppState::default();
        initial.set_tool_permission_context(crate::tool::ToolPermissionContext {
            mode: PermissionMode::AcceptEdits,
            ..Default::default()
        });
        let store = AppStore::new(initial, None);
        let context = crate::tool::ToolUseContext::default().with_app_store(store.clone());

        enter_plan_mode_call(&context).expect("enter should succeed");
        let state = store.get();
        assert_eq!(state.tool_permission_context.mode, PermissionMode::Plan);
        assert_eq!(
            state.tool_permission_context.pre_plan_mode,
            Some(PermissionMode::AcceptEdits)
        );
    }
}
