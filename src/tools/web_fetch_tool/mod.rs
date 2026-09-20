//! WebFetch tool metadata and UI.
//!
//! Maps to:
//! - CC `tools/WebFetchTool/WebFetchTool.ts`
//! - CC `tools/WebFetchTool/prompt.ts`
//! - CC `tools/WebFetchTool/UI.tsx`
//!
//! Execution lives in this module, dispatched from `services/tools/tool_execution.rs`.

pub mod preapproved;
pub mod prompt;
pub mod ui;
pub mod utils;

/// Maps to CC `WebFetchTool.inputSchema`.
/// Maps to: CC `WebFetchTool.ts:24-29` `inputSchema`.
pub fn input_schema() -> &'static crate::utils::zod::Schema {
    static SCHEMA: std::sync::OnceLock<crate::utils::zod::Schema> = std::sync::OnceLock::new();
    SCHEMA.get_or_init(|| {
        use crate::utils::zod;
        zod::strict_object(vec![
            (
                "url",
                zod::string()
                    .url()
                    .describe("The URL to fetch content from"),
            ),
            (
                "prompt",
                zod::string().describe("The prompt to run on the fetched content"),
            ),
        ])
    })
}

/// Maps to: CC `WebFetchTool.ts:50-64`
/// `webFetchToolInputToPermissionRuleContent(input)`. The failure arm is
/// CC's `` `input:${input.toString()}` `` — a plain object stringifies to
/// "[object Object]", and stored permission rules match on that exact text.
fn web_fetch_tool_input_to_permission_rule_content(input: &serde_json::Value) -> String {
    input
        .get("url")
        .and_then(|value| value.as_str())
        .and_then(preapproved::parse_web_fetch_url_host_path)
        .map(|(hostname, _)| format!("domain:{hostname}"))
        .unwrap_or_else(|| "input:[object Object]".to_string())
}

pub fn web_fetch_tool_schema() -> crate::types::tools::Tool {
    crate::types::tools::Tool {
        name: prompt::WEB_FETCH_TOOL_NAME.to_string(),
        description: prompt::tool_prompt().to_string(),
        input_schema: crate::utils::zod_to_json_schema::zod_to_json_schema(input_schema()),
        ..Default::default()
    }
}

/// Maps to: CC `tools/WebFetchTool/WebFetchTool.ts:48` `export type Output =
/// z.infer<OutputSchema>` (schema at :32-45) — the single type the tool
/// yields from `call()`, records as the message's `toolUseResult`, and the
/// render path recovers via `outputSchema.safeParse` ([`ui::parse_output`] is
/// the Rust stand-in).
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub bytes: usize,
    pub code: i64,
    pub code_text: String,
    pub result: String,
    pub duration_ms: u64,
    pub url: String,
}

fn web_fetch_error_output(message: impl Into<String>) -> crate::tool::ToolOutput {
    let message = message.into();
    // The `Error: …` raw string rides the row via the `tool_use_result`
    // trait projection, not a display variant.
    crate::tool::ToolOutput::Composed {
        content: format!("<tool_use_error>{message}</tool_use_error>"),
        status: crate::types::message::ToolResultStatus::Error,
    }
}

fn web_fetch_redirect_output(
    url: &str,
    prompt: &str,
    redirect: utils::RedirectInfo,
    duration_ms: u64,
) -> Output {
    // Maps to CC `tools/WebFetchTool/WebFetchTool.ts` redirect branch in `call`.
    let result = utils::redirect_message(&redirect, prompt);
    Output {
        bytes: result.len(),
        code: i64::from(redirect.status_code),
        code_text: utils::redirect_code_text(redirect.status_code),
        result,
        duration_ms,
        url: url.to_string(),
    }
}

fn preapproved_markdown_direct_result(
    url: &str,
    content_type: &str,
    content: &str,
) -> Option<String> {
    // Maps to CC `WebFetchTool.call` preapproved text/markdown fast path.
    if utils::is_preapproved_url(url)
        && content_type.contains("text/markdown")
        && content.len() < utils::MAX_MARKDOWN_LENGTH
    {
        Some(content.to_string())
    } else {
        None
    }
}

fn append_binary_content_note(
    result: &mut String,
    content_type: &str,
    persisted_path: Option<&str>,
    persisted_size: Option<usize>,
    bytes: usize,
) {
    // Maps to CC `WebFetchTool.call` persistedPath result suffix.
    if let Some(path) = persisted_path {
        let size = persisted_size.unwrap_or(bytes);
        result.push_str(&format!(
            "\n\n[Binary content ({content_type}, {}) also saved to {path}]",
            crate::utils::format::format_file_size(size as u64)
        ));
    }
}

/// Behavioral half of CC `WebFetchTool` — dispatched via `crate::tool::ToolCall`.
pub(crate) struct WebFetchTool;

impl crate::tool::ToolCall for WebFetchTool {
    fn name(&self) -> &'static str {
        "WebFetch"
    }

    /// Maps to: CC `WebFetchTool.ts:181-191` `async prompt(_options)` — a
    /// constant template (auth warning + DESCRIPTION) that deliberately
    /// IGNORES the options bag: CC's in-source comment explains that toggling
    /// the warning on ToolSearch availability flickered the description
    /// between SDK query() calls and busted the prompt cache twice per
    /// flicker. Same source the wire schema renders eagerly.
    fn prompt(
        &self,
        _tool: &crate::types::tools::Tool,
        _options: &crate::tool::ToolPromptOptions<'_>,
    ) -> String {
        prompt::tool_prompt().to_string()
    }

    /// Maps to: CC `WebFetchTool.isConcurrencySafe(...)` read-only default.
    fn is_concurrency_safe(&self, _args: &serde_json::Value) -> bool {
        true
    }

    /// Maps to: CC `WebFetchTool.isReadOnly()` (:98-100).
    fn is_read_only(&self, _args: &serde_json::Value) -> bool {
        true
    }

    /// Maps to: CC `WebFetchTool.searchHint` (:68).
    fn search_hint(&self) -> Option<&'static str> {
        Some("fetch and extract content from a URL")
    }

    /// Maps to: CC `WebFetchTool.ts:11,84` mounting `UI.tsx#getToolUseSummary`.
    fn get_tool_use_summary(&self, args: &serde_json::Value) -> Option<String> {
        crate::tools::web_fetch_tool::ui::get_tool_use_summary(Some(args))
    }

    /// Maps to: CC `WebFetchTool.ts:85-88` `getActivityDescription(input)`.
    fn get_activity_description(&self, args: &serde_json::Value) -> Option<String> {
        Some(
            match crate::tools::web_fetch_tool::ui::get_tool_use_summary(Some(args)) {
                Some(summary) if !summary.is_empty() => format!("Fetching {summary}"),
                _ => "Fetching web page".to_string(),
            },
        )
    }

    /// Maps to: CC `WebFetchTool.description(input)` (:72-80).
    fn description(&self, args: &serde_json::Value) -> String {
        args.get("url")
            .and_then(|value| value.as_str())
            .and_then(preapproved::parse_web_fetch_url_host_path)
            .map_or_else(
                || "Claude wants to fetch content from this URL".to_string(),
                |(hostname, _)| format!("Claude wants to fetch content from {hostname}"),
            )
    }

    /// Maps to: CC `WebFetchTool.shouldDefer` (:71).
    fn should_defer(&self) -> bool {
        true
    }

    /// Maps to: CC `WebFetchTool.maxResultSizeChars` (:70).
    fn max_result_size_chars(&self) -> usize {
        100_000
    }

    /// Maps to: CC `WebFetchTool.userFacingName()` (:81-83).
    fn user_facing_name(&self, _args: Option<&serde_json::Value>) -> String {
        "Fetch".to_string()
    }

    /// Maps to: CC `WebFetchTool.toAutoClassifierInput(input)` (:101-103).
    fn to_auto_classifier_input(&self, args: &serde_json::Value) -> String {
        let url = args
            .get("url")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        match args.get("prompt").and_then(|value| value.as_str()) {
            Some(prompt) if !prompt.is_empty() => format!("{url}: {prompt}"),
            _ => url.to_string(),
        }
    }

    /// Maps to: CC `WebFetchTool.validateInput(input)` (:191-204).
    fn validate_input(
        &self,
        args: &serde_json::Value,
        _context: &crate::tool::ToolUseContext,
    ) -> crate::tool::ValidationResult {
        let url = args
            .get("url")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        if utils::is_parseable_url(url) {
            crate::tool::ValidationResult::Ok
        } else {
            crate::tool::ValidationResult::error(
                format!("Error: Invalid URL \"{url}\". The URL provided could not be parsed."),
                1,
            )
        }
    }

    /// Maps to: CC `WebFetchTool.checkPermissions(input, context)` (:104-180).
    fn check_permissions(
        &self,
        args: &serde_json::Value,
        context: &crate::tool::ToolUseContext,
    ) -> crate::utils::permissions::permission_result::PermissionResult {
        use crate::types::permissions::{
            PermissionBehavior, PermissionRuleValue, PermissionUpdate, PermissionUpdateDestination,
        };
        use crate::utils::permissions::permission_result::{
            PermissionDecisionReason, PermissionResult,
        };

        let permission_context = &context.tool_permission_context;

        if let Some((hostname, pathname)) = args
            .get("url")
            .and_then(|value| value.as_str())
            .and_then(preapproved::parse_web_fetch_url_host_path)
        {
            if preapproved::is_preapproved_host(&hostname, &pathname) {
                return PermissionResult::Allow {
                    updated_input: Some(args.clone()),
                    user_modified: None,
                    decision_reason: Some(PermissionDecisionReason::Other {
                        reason: "Preapproved host".to_string(),
                    }),
                    tool_use_id: None,
                    accept_feedback: None,
                    content_blocks: Vec::new(),
                };
            }
        }

        let rule_content = web_fetch_tool_input_to_permission_rule_content(args);
        let suggestions = || {
            vec![PermissionUpdate::AddRules {
                destination: PermissionUpdateDestination::LocalSettings,
                behavior: PermissionBehavior::Allow,
                rules: vec![PermissionRuleValue::new(
                    prompt::WEB_FETCH_TOOL_NAME,
                    Some(rule_content.clone()),
                )],
            }]
        };
        let rule_for = |behavior: PermissionBehavior| {
            crate::utils::permissions::permissions::get_rule_by_contents_for_tool_name(
                permission_context,
                prompt::WEB_FETCH_TOOL_NAME,
                behavior,
            )
            .get(&rule_content)
            .cloned()
        };

        if let Some(rule) = rule_for(PermissionBehavior::Deny) {
            return PermissionResult::Deny {
                message: format!(
                    "{} denied access to {rule_content}.",
                    prompt::WEB_FETCH_TOOL_NAME
                ),
                decision_reason: PermissionDecisionReason::Rule { rule },
                tool_use_id: None,
            };
        }
        if let Some(rule) = rule_for(PermissionBehavior::Ask) {
            return PermissionResult::Ask {
                message: format!(
                    "Claude requested permissions to use {}, but you haven't granted it yet.",
                    prompt::WEB_FETCH_TOOL_NAME
                ),
                updated_input: None,
                decision_reason: Some(PermissionDecisionReason::Rule { rule }),
                suggestions: suggestions(),
                blocked_path: None,
                metadata: None,
                is_bash_security_check_for_misparsing: false,
                pending_classifier_check: None,
                content_blocks: Vec::new(),
            };
        }
        if let Some(rule) = rule_for(PermissionBehavior::Allow) {
            return PermissionResult::Allow {
                updated_input: Some(args.clone()),
                user_modified: None,
                decision_reason: Some(PermissionDecisionReason::Rule { rule }),
                tool_use_id: None,
                accept_feedback: None,
                content_blocks: Vec::new(),
            };
        }

        PermissionResult::Ask {
            message: format!(
                "Claude requested permissions to use {}, but you haven't granted it yet.",
                prompt::WEB_FETCH_TOOL_NAME
            ),
            updated_input: None,
            decision_reason: None,
            suggestions: suggestions(),
            blocked_path: None,
            metadata: None,
            is_bash_security_check_for_misparsing: false,
            pending_classifier_check: None,
            content_blocks: Vec::new(),
        }
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
            let _ = request;
            let url = args
                .get("url")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let prompt = args
                .get("prompt")
                .and_then(|value| value.as_str())
                .unwrap_or_default();

            let start = std::time::Instant::now();
            let data = match utils::get_url_markdown_content(url, &context.abort_controller).await {
                Ok(utils::UrlMarkdownContent::Redirect(redirect)) => {
                    crate::tool::ToolOutput::WebFetch(web_fetch_redirect_output(
                        url,
                        prompt,
                        redirect,
                        start.elapsed().as_millis() as u64,
                    ))
                }
                Ok(utils::UrlMarkdownContent::Fetched(content)) => {
                    let result = if let Some(result) = preapproved_markdown_direct_result(
                        url,
                        &content.content_type,
                        &content.content,
                    ) {
                        Ok(result)
                    } else {
                        utils::apply_prompt_to_markdown(
                            prompt,
                            &content.content,
                            &context.abort_controller,
                            context.is_non_interactive_session,
                            utils::is_preapproved_url(url),
                        )
                        .await
                    };

                    match result {
                        Ok(mut result) => {
                            append_binary_content_note(
                                &mut result,
                                &content.content_type,
                                content.persisted_path.as_deref(),
                                content.persisted_size,
                                content.bytes,
                            );
                            crate::tool::ToolOutput::WebFetch(Output {
                                bytes: content.bytes,
                                code: i64::from(content.code),
                                code_text: content.code_text,
                                result,
                                duration_ms: start.elapsed().as_millis() as u64,
                                url: url.to_string(),
                            })
                        }
                        Err(error) => web_fetch_error_output(format!("Error: {error}")),
                    }
                }
                Err(error) => web_fetch_error_output(format!("Error: {error}")),
            };

            crate::tool::ToolResult {
                data,
                new_messages: Vec::new(),
            }
        })
    }

    /// Maps to: CC `tools/WebFetchTool/WebFetchTool.ts`
    /// `mapToolResultToToolResultBlockParam` (:303-309).
    fn map_tool_result_to_tool_result_block_param(
        &self,
        data: &crate::tool::ToolOutput,
        _tool_use_id: &str,
    ) -> (String, crate::types::message::ToolResultStatus) {
        match data {
            crate::tool::ToolOutput::WebFetch(output) => (
                output.result.clone(),
                crate::types::message::ToolResultStatus::Success,
            ),
            crate::tool::ToolOutput::Composed {
                content, status, ..
            } => (content.clone(), *status),
            _ => (
                "<tool_use_error>WebFetch returned an unexpected output variant</tool_use_error>"
                    .to_string(),
                crate::types::message::ToolResultStatus::Error,
            ),
        }
    }

    /// WebFetch carries no display shape — the raw the trait projects
    /// below is what renders (`renderToolResultMessage` parses it with the
    /// tool's own output schema). A live `Output` is typed, so its projection
    /// always parses; the emit gate never fires here, unlike the cold paths
    /// that must gate on `ui::parse_output`.
    /// Maps to: CC recording WebFetchTool's `Output` as the message's
    /// `toolUseResult`. A failure records the `Error: …` string, matching the
    /// string `toolUseResult` CC keeps for errored fetches.
    fn tool_use_result(&self, data: &crate::tool::ToolOutput) -> Option<serde_json::Value> {
        match data {
            crate::tool::ToolOutput::WebFetch(output) => {
                Some(crate::tools::web_fetch_tool::ui::output_to_value(output))
            }
            crate::tool::ToolOutput::Composed {
                content,
                status: crate::types::message::ToolResultStatus::Error,
                ..
            } => {
                // The Composed content carries the model-facing tag; CC's raw
                // string is the bare `Error: …` message.
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
    fn web_fetch_tool_schema_matches_official_input_shape() {
        let schema = web_fetch_tool_schema();
        assert_eq!(schema.name, "WebFetch");
        assert_eq!(
            schema.input_schema["required"],
            serde_json::json!(["url", "prompt"])
        );
        assert_eq!(schema.input_schema["properties"]["url"]["format"], "uri");
        assert!(schema.description.contains("Fetches content"));
        assert!(
            schema
                .description
                .starts_with("IMPORTANT: WebFetch WILL FAIL for authenticated or private URLs.")
        );

        let secondary = prompt::make_secondary_model_prompt("# Doc", "Summarize", false);
        assert!(secondary.contains("Web page content"));
        assert!(secondary.contains("125-character maximum"));
    }

    #[test]
    fn web_fetch_preapproved_markdown_fast_path_matches_official_branch() {
        assert_eq!(
            preapproved_markdown_direct_result(
                "https://doc.rust-lang.org/book/",
                "text/markdown; charset=utf-8",
                "# Rust Book"
            ),
            Some("# Rust Book".to_string())
        );
        assert!(
            preapproved_markdown_direct_result(
                "https://example.com/docs",
                "text/markdown",
                "# Example"
            )
            .is_none()
        );
        assert!(
            preapproved_markdown_direct_result(
                "https://doc.rust-lang.org/book/",
                "text/html",
                "# Rust Book"
            )
            .is_none()
        );
        assert!(
            preapproved_markdown_direct_result(
                "https://doc.rust-lang.org/book/",
                "text/markdown",
                &"a".repeat(utils::MAX_MARKDOWN_LENGTH)
            )
            .is_none()
        );
    }

    #[test]
    fn web_fetch_binary_persistence_suffix_matches_official_copy() {
        let mut result = "summary".to_string();
        append_binary_content_note(
            &mut result,
            "application/pdf",
            Some("/tmp/webfetch.pdf"),
            Some(2048),
            4096,
        );
        assert_eq!(
            result,
            "summary\n\n[Binary content (application/pdf, 2KB) also saved to /tmp/webfetch.pdf]"
        );

        let mut no_path = "summary".to_string();
        append_binary_content_note(&mut no_path, "application/pdf", None, Some(2048), 4096);
        assert_eq!(no_path, "summary");
    }

    #[test]
    fn web_fetch_redirect_output_returns_official_success_schema() {
        use crate::tool::ToolCall;

        let tool = WebFetchTool;
        let output = web_fetch_redirect_output(
            "https://example.com/docs",
            "summarize",
            utils::RedirectInfo {
                original_url: "https://example.com/docs".to_string(),
                redirect_url: "https://other.example/docs".to_string(),
                status_code: 308,
            },
            12,
        );
        assert_eq!(output.bytes, output.result.len());
        assert_eq!(output.code, 308);
        assert_eq!(output.code_text, "Permanent Redirect");
        assert_eq!(output.duration_ms, 12);
        assert_eq!(output.url, "https://example.com/docs");
        assert!(output.result.contains("REDIRECT DETECTED"));

        let data = crate::tool::ToolOutput::WebFetch(output);
        let (content, status) =
            tool.map_tool_result_to_tool_result_block_param(&data, "toolu_web_fetch");
        assert_eq!(status, crate::types::message::ToolResultStatus::Success);
        assert!(content.contains("Please use WebFetch again"));
        // No WebFetch display shape — the trait projects the raw
        // `toolUseResult` with every schema field intact.
        let raw = tool
            .tool_use_result(&data)
            .expect("web fetch projects raw output");
        assert_eq!(raw["code"], serde_json::json!(308));
        assert_eq!(raw["codeText"], serde_json::json!("Permanent Redirect"));
        assert!(
            raw["result"]
                .as_str()
                .is_some_and(|result| result.contains("REDIRECT DETECTED"))
        );
        // The live projection always satisfies the tool's own output schema.
        assert!(crate::tools::web_fetch_tool::ui::parse_output(&raw).is_some());
    }

    #[test]
    fn web_fetch_auto_classifier_input_matches_official_projection() {
        use crate::tool::ToolCall;

        let tool = WebFetchTool;
        assert_eq!(
            tool.to_auto_classifier_input(&serde_json::json!({
                "url": "https://example.com/docs",
                "prompt": "summarize"
            })),
            "https://example.com/docs: summarize"
        );
        assert_eq!(
            tool.to_auto_classifier_input(&serde_json::json!({
                "url": "https://example.com/docs"
            })),
            "https://example.com/docs"
        );
    }

    #[test]
    fn web_fetch_validate_input_matches_official_invalid_url_copy() {
        use crate::tool::ToolCall;

        let tool = WebFetchTool;
        let context = crate::tool::ToolUseContext::default();
        assert!(
            tool.validate_input(
                &serde_json::json!({"url": "https://example.com", "prompt": "p"}),
                &context
            )
            .is_ok()
        );
        assert_eq!(
            tool.validate_input(
                &serde_json::json!({"url": "not a url", "prompt": "p"}),
                &context
            ),
            crate::tool::ValidationResult::error(
                "Error: Invalid URL \"not a url\". The URL provided could not be parsed.",
                1
            )
        );
    }

    #[test]
    fn web_fetch_check_permissions_deny_uses_official_copy() {
        use crate::tool::ToolCall;
        use crate::types::permissions::{PermissionRuleSource, PermissionRuleValue};

        let mut context = crate::tool::ToolUseContext::default();
        context.tool_permission_context.always_deny_rules.insert(
            PermissionRuleSource::Session,
            vec![PermissionRuleValue::new(
                "WebFetch",
                Some("domain:example.com".to_string()),
            )],
        );
        let decision = WebFetchTool.check_permissions(
            &serde_json::json!({"url": "https://example.com/docs", "prompt": "p"}),
            &context,
        );
        assert!(matches!(
            decision,
            crate::utils::permissions::permission_result::PermissionResult::Deny { message, .. }
            if message == "WebFetch denied access to domain:example.com."
        ));
    }

    #[tokio::test]
    async fn web_fetch_tool_call_invalid_url_returns_error_without_network() {
        use crate::tool::ToolCall;

        let args = serde_json::json!({
            "url": "not a url",
            "prompt": "summarize"
        });
        let request = crate::utils::permissions::permissions::mock_permission_request_with_input(
            "perm-web-fetch".to_string(),
            "toolu_web_fetch".to_string(),
            "WebFetch".to_string(),
            "input:not a url".to_string(),
            args.clone(),
            crate::types::permissions::PermissionMode::Default,
        );
        let tool = WebFetchTool;
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

        let (content, status) =
            tool.map_tool_result_to_tool_result_block_param(&result.data, "toolu_web_fetch");
        assert_eq!(status, crate::types::message::ToolResultStatus::Error);
        assert!(content.contains("Invalid URL"));
        // The error rides the row as the bare `Error: …` raw string.
        assert!(matches!(
            tool.tool_use_result(&result.data),
            Some(serde_json::Value::String(raw)) if raw.contains("Invalid URL")
        ));
    }
}
