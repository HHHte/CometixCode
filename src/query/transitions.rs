//! Query loop transition types.
//! Maps to official `query/transitions.ts`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terminal {
    pub reason: String,
    /// Rust actor carrier for a thrown query error (distinct from a yielded API error).
    pub exception: Option<String>,
}

impl Terminal {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            exception: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Continue {
    pub reason: Option<String>,
}

impl Continue {
    pub fn new() -> Self {
        Self { reason: None }
    }

    pub fn with_reason(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
        }
    }

    /// Rust control-flow helper for sites where CC `query.ts` assigns a
    /// `{ type: 'continue' }` transition and immediately advances the query
    /// loop with updated model history.
    pub fn should_continue_query_loop(self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_and_continue_match_official_shape() {
        assert_eq!(Terminal::new("completed").reason, "completed");
        assert_eq!(Continue::new().reason, None);
        assert_eq!(
            Continue::with_reason("next_turn").reason.as_deref(),
            Some("next_turn")
        );
        assert!(Continue::new().should_continue_query_loop());
    }

    #[test]
    fn query_terminal_literal_reasons_use_official_query_names() {
        let query_rs = include_str!("../query.rs");
        let official = [
            "completed",
            "model_error",
            "aborted_streaming",
            "aborted_tools",
            "blocking_limit",
            "collapse_drain_retry",
            "hook_stopped",
            "image_error",
            "max_output_tokens_escalate",
            "max_output_tokens_recovery",
            "max_turns",
            "next_turn",
            "prompt_too_long",
            "reactive_compact_retry",
            "stop_hook_blocking",
            "stop_hook_prevented",
            "token_budget_continuation",
        ];
        let mut remaining = query_rs;
        while let Some(idx) = remaining.find("Terminal::new(") {
            remaining = &remaining[idx + "Terminal::new(".len()..];
            let trimmed = remaining.trim_start();
            if let Some(after_quote) = trimmed.strip_prefix('"') {
                let Some(end) = after_quote.find('"') else {
                    panic!("unterminated Terminal::new literal");
                };
                let reason = &after_quote[..end];
                assert!(
                    official.contains(&reason),
                    "non-official query terminal reason literal: {reason}"
                );
                remaining = &after_quote[end + 1..];
            }
        }

        let mut remaining = query_rs;
        while let Some(idx) = remaining.find("Continue::with_reason(") {
            remaining = &remaining[idx + "Continue::with_reason(".len()..];
            let trimmed = remaining.trim_start();
            if let Some(after_quote) = trimmed.strip_prefix('"') {
                let Some(end) = after_quote.find('"') else {
                    panic!("unterminated Continue::with_reason literal");
                };
                let reason = &after_quote[..end];
                assert!(
                    official.contains(&reason),
                    "non-official query continue reason literal: {reason}"
                );
                remaining = &after_quote[end + 1..];
            }
        }

        let mut remaining = query_rs;
        while let Some(idx) = remaining.find("\"aborted_") {
            remaining = &remaining[idx + 1..];
            let Some(end) = remaining.find('"') else {
                panic!("unterminated aborted_* literal");
            };
            let reason = &remaining[..end];
            assert!(
                matches!(reason, "aborted_streaming" | "aborted_tools"),
                "non-official aborted query reason literal: {reason}"
            );
            remaining = &remaining[end + 1..];
        }
    }

    #[test]
    fn production_query_core_keeps_permission_and_tool_result_ownership_outside_query_rs() {
        let query_rs = include_str!("../query.rs");
        let production_query_rs = query_rs
            .split("#[cfg(test)]\nmod tests")
            .next()
            .expect("query.rs should have production section");

        assert!(
            !production_query_rs.contains("PermissionRequest {")
                && !production_query_rs.contains("PermissionRequest::"),
            "query.rs production code must not construct PermissionRequest; permission flow belongs to tool execution/useCanUseTool"
        );
        assert!(
            !production_query_rs.contains("crate::types::message::ToolResult {")
                && !production_query_rs.contains("types::message::ToolResult {")
                && !production_query_rs.contains("ToolResult::"),
            "query.rs production code must not construct tool_result blocks; tool execution owns tool_result creation"
        );
        assert!(
            !production_query_rs.contains("query_model_with_streaming"),
            "query.rs production code must call the model only through QueryDeps::call_model"
        );
    }

    #[test]
    fn query_layout_keeps_only_official_query_modules_and_no_legacy_tool_context_files() {
        let expected = [
            "config.rs",
            "deps.rs",
            // PORTING.md A2/A6: Rust transport for generator next()/return().
            "event_channel.rs",
            "stop_hooks.rs",
            "token_budget.rs",
            "transitions.rs",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<std::collections::BTreeSet<_>>();
        let actual = std::fs::read_dir("src/query")
            .expect("read src/query")
            .filter_map(|entry| {
                let entry = entry.expect("read src/query entry");
                entry
                    .file_type()
                    .expect("read src/query file type")
                    .is_file()
                    .then(|| entry.file_name().to_string_lossy().to_string())
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            actual, expected,
            "src/query must stay aligned to official query/* modules only"
        );
        assert!(
            include_str!("../query.rs")
                .contains("#[cfg(test)]\nmod prompt_context_contract_tests {")
        );
        assert!(
            !std::path::Path::new("src/services/tools/tool_context.rs").exists(),
            "ToolUseContext belongs in src/tool.rs; do not restore legacy tool_context.rs"
        );
        assert!(
            !std::path::Path::new("src/services/tools/tool_executor.rs").exists(),
            "do not restore legacy tool_executor.rs boundary"
        );
    }
}
