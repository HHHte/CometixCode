//! Diagnostic tracking formatting helpers.
//!
//! Maps to: CC `services/diagnosticTracking.ts` `DiagnosticTrackingService`.
//! Owns the IDE/MCP baseline, right-file state, query reset, new-diagnostic
//! filtering, and shared UI/model summary formatting.

use crate::services::lsp::types::{Diagnostic, DiagnosticFile};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const MAX_DIAGNOSTICS_SUMMARY_CHARS: usize = 4000;

#[derive(Default)]
struct DiagnosticTrackerState {
    baseline: HashMap<String, Vec<Diagnostic>>,
    right_file_diagnostics_state: HashMap<String, Vec<Diagnostic>>,
    last_processed_timestamps: HashMap<String, i64>,
}

static TRACKER: LazyLock<Mutex<DiagnosticTrackerState>> =
    LazyLock::new(|| Mutex::new(DiagnosticTrackerState::default()));
const TRUNCATION_MARKER: &str = "…[truncated]";

fn normalize_file_uri(file_uri: &str) -> String {
    let without_protocol = ["file://", "_claude_fs_right:", "_claude_fs_left:"]
        .into_iter()
        .find_map(|prefix| file_uri.strip_prefix(prefix))
        .unwrap_or(file_uri);
    let mut normalized = std::path::Path::new(without_protocol)
        .components()
        .fold(std::path::PathBuf::new(), |mut path, component| {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    path.pop();
                }
                other => path.push(other.as_os_str()),
            }
            path
        })
        .display()
        .to_string();
    if cfg!(windows) {
        normalized = normalized.replace('/', "\\").to_lowercase();
    }
    normalized
}

fn parse_diagnostic_result(result: &serde_json::Value) -> Vec<DiagnosticFile> {
    let blocks = result
        .as_array()
        .or_else(|| result.get("content").and_then(serde_json::Value::as_array));
    let Some(text) = blocks.and_then(|blocks| {
        blocks.iter().find_map(|block| {
            (block.get("type").and_then(serde_json::Value::as_str) == Some("text"))
                .then(|| block.get("text").and_then(serde_json::Value::as_str))
                .flatten()
        })
    }) else {
        return Vec::new();
    };
    serde_json::from_str(text).unwrap_or_default()
}

/// Maps to CC `DiagnosticTrackingService.beforeFileEdited(filePath)`.
///
/// Rust adaptation: MCP connections live in the process-global runtime rather
/// than an object cached by `handleQueryStart`, so connected-IDE lookup happens
/// at call time (also avoiding CC's stale-client TODO).
pub async fn before_file_edited(file_path: &str) {
    if !crate::services::mcp::client::is_connected_mcp_client("ide").await {
        return;
    }
    let timestamp = chrono::Utc::now().timestamp_millis();
    let mut arguments = serde_json::Map::new();
    arguments.insert(
        "uri".to_string(),
        serde_json::Value::String(format!("file://{file_path}")),
    );
    let Ok(result) =
        crate::services::mcp::client::call_mcp_tool("ide", "getDiagnostics", arguments).await
    else {
        return;
    };
    let files = parse_diagnostic_result(&result);
    let expected = normalize_file_uri(file_path);
    let diagnostics = match files.first() {
        Some(file) if normalize_file_uri(&file.uri) == expected => file.diagnostics.clone(),
        Some(file) => {
            crate::utils::debug::log_for_debugging(&format!(
                "Diagnostics file path mismatch: expected {file_path}, got {})",
                file.uri
            ));
            return;
        }
        None => Vec::new(),
    };
    let mut tracker = TRACKER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    tracker.baseline.insert(expected.clone(), diagnostics);
    tracker
        .last_processed_timestamps
        .insert(expected, timestamp);
}

fn diagnostic_arrays_equal(left: &[Diagnostic], right: &[Diagnostic]) -> bool {
    left.len() == right.len()
        && left.iter().all(|diagnostic| right.contains(diagnostic))
        && right.iter().all(|diagnostic| left.contains(diagnostic))
}

fn collect_new_diagnostics(
    tracker: &mut DiagnosticTrackerState,
    files: &[DiagnosticFile],
) -> Vec<DiagnosticFile> {
    let right_files = files
        .iter()
        .filter(|file| file.uri.starts_with("_claude_fs_right:"))
        .filter(|file| {
            tracker
                .baseline
                .contains_key(&normalize_file_uri(&file.uri))
        })
        .map(|file| (normalize_file_uri(&file.uri), file))
        .collect::<HashMap<_, _>>();
    let tracked_file_uris = files
        .iter()
        .filter(|file| file.uri.starts_with("file://"))
        .filter(|file| {
            tracker
                .baseline
                .contains_key(&normalize_file_uri(&file.uri))
        })
        .collect::<Vec<_>>();
    let mut new_files = Vec::new();
    for file in tracked_file_uris {
        let normalized = normalize_file_uri(&file.uri);
        let baseline = tracker
            .baseline
            .get(&normalized)
            .cloned()
            .unwrap_or_default();
        let selected = if let Some(right) = right_files.get(&normalized) {
            let changed = tracker
                .right_file_diagnostics_state
                .get(&normalized)
                .is_none_or(|previous| !diagnostic_arrays_equal(previous, &right.diagnostics));
            tracker
                .right_file_diagnostics_state
                .insert(normalized.clone(), right.diagnostics.clone());
            if changed { *right } else { file }
        } else {
            file
        };
        let diagnostics = selected
            .diagnostics
            .iter()
            .filter(|diagnostic| !baseline.contains(diagnostic))
            .cloned()
            .collect::<Vec<_>>();
        if !diagnostics.is_empty() {
            new_files.push(DiagnosticFile {
                uri: file.uri.clone(),
                diagnostics,
            });
        }
        tracker
            .baseline
            .insert(normalized, selected.diagnostics.clone());
    }
    new_files
}

/// Maps to CC `DiagnosticTrackingService.getNewDiagnostics()`.
pub async fn get_new_diagnostics() -> Vec<DiagnosticFile> {
    if !crate::services::mcp::client::is_connected_mcp_client("ide").await {
        return Vec::new();
    }
    let Ok(result) = crate::services::mcp::client::call_mcp_tool(
        "ide",
        "getDiagnostics",
        serde_json::Map::new(),
    )
    .await
    else {
        return Vec::new();
    };
    let files = parse_diagnostic_result(&result);
    let mut tracker = TRACKER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    collect_new_diagnostics(&mut tracker, &files)
}

/// Maps to CC `DiagnosticTrackingService.reset()`.
pub fn reset() {
    *TRACKER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = DiagnosticTrackerState::default();
}

/// Maps to: CC `DiagnosticTrackingService.getSeveritySymbol(...)`.
pub fn get_severity_symbol(severity: &str) -> &'static str {
    let figures = crate::constants::figures::figures();
    match severity {
        "Error" => figures.cross,
        "Warning" => figures.warning,
        "Info" => figures.info,
        "Hint" => figures.star,
        _ => figures.bullet,
    }
}

/// Maps to: CC `DiagnosticTrackingService.formatDiagnosticsSummary(...)`.
pub fn format_diagnostics_summary(files: &[DiagnosticFile]) -> String {
    let result = files
        .iter()
        .map(|file| {
            let filename = file.uri.rsplit('/').next().unwrap_or(&file.uri);
            let diagnostics = file
                .diagnostics
                .iter()
                .map(format_single_diagnostic)
                .collect::<Vec<_>>()
                .join("\n");
            format!("{filename}:\n{diagnostics}")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    if result.chars().count() > MAX_DIAGNOSTICS_SUMMARY_CHARS {
        let take = MAX_DIAGNOSTICS_SUMMARY_CHARS.saturating_sub(TRUNCATION_MARKER.chars().count());
        format!(
            "{}{}",
            result.chars().take(take).collect::<String>(),
            TRUNCATION_MARKER
        )
    } else {
        result
    }
}

/// Maps to: CC `components/DiagnosticsDisplay.tsx` normal-mode summary text.
pub fn format_diagnostics_display_summary(files: &[DiagnosticFile]) -> String {
    let total_issues = files
        .iter()
        .map(|file| file.diagnostics.len())
        .sum::<usize>();
    let file_count = files.len();
    let issue_word = if total_issues == 1 { "issue" } else { "issues" };
    let file_word = if file_count == 1 { "file" } else { "files" };
    format!("Found {total_issues} new diagnostic {issue_word} in {file_count} {file_word}")
}

/// Maps to each diagnostic row in CC `components/DiagnosticsDisplay.tsx`.
pub fn format_single_diagnostic(diagnostic: &Diagnostic) -> String {
    let code = diagnostic
        .code
        .as_ref()
        .map(|code| format!(" [{code}]"))
        .unwrap_or_default();
    let source = diagnostic
        .source
        .as_ref()
        .map(|source| format!(" ({source})"))
        .unwrap_or_default();
    format!(
        "  {} [Line {}:{}] {}{}{}",
        get_severity_symbol(&diagnostic.severity),
        diagnostic.range.start.line + 1,
        diagnostic.range.start.character + 1,
        diagnostic.message,
        code,
        source
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::lsp::types::{DiagnosticPosition, DiagnosticRange};

    fn diag(message: &str, severity: &str, line: u32) -> Diagnostic {
        Diagnostic {
            message: message.to_string(),
            severity: severity.to_string(),
            range: DiagnosticRange {
                start: DiagnosticPosition { line, character: 2 },
                end: DiagnosticPosition { line, character: 8 },
            },
            source: Some("rust-analyzer".to_string()),
            code: Some("E0425".to_string()),
        }
    }

    #[test]
    fn diagnostic_result_parser_reads_first_text_block() {
        let value = serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::to_string(&vec![DiagnosticFile {
                    uri: "file:///tmp/a.rs".to_string(),
                    diagnostics: vec![diag("broken", "Error", 0)],
                }]).unwrap()
            }]
        });
        let parsed = parse_diagnostic_result(&value);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].diagnostics[0].message, "broken");
    }

    #[test]
    fn new_diagnostics_are_diffed_against_baseline_and_then_advanced() {
        let old = diag("old", "Warning", 0);
        let new = diag("new", "Error", 1);
        let mut tracker = DiagnosticTrackerState::default();
        tracker
            .baseline
            .insert("/tmp/a.rs".to_string(), vec![old.clone()]);
        let files = vec![DiagnosticFile {
            uri: "file:///tmp/a.rs".to_string(),
            diagnostics: vec![old, new.clone()],
        }];
        let first = collect_new_diagnostics(&mut tracker, &files);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].diagnostics, vec![new]);
        assert!(collect_new_diagnostics(&mut tracker, &files).is_empty());
    }

    #[test]
    fn right_file_diagnostics_follow_official_change_selection_state() {
        let baseline = diag("baseline", "Warning", 0);
        let disk_new = diag("disk", "Error", 1);
        let right_new = diag("right", "Hint", 2);
        let right_changed = diag("right changed", "Info", 3);
        let mut tracker = DiagnosticTrackerState::default();
        tracker
            .baseline
            .insert("/tmp/a.rs".to_string(), vec![baseline.clone()]);

        let files = |right: Diagnostic| {
            vec![
                DiagnosticFile {
                    uri: "file:///tmp/a.rs".to_string(),
                    diagnostics: vec![baseline.clone(), disk_new.clone()],
                },
                DiagnosticFile {
                    uri: "_claude_fs_right:/tmp/a.rs".to_string(),
                    diagnostics: vec![baseline.clone(), right],
                },
            ]
        };

        let first = collect_new_diagnostics(&mut tracker, &files(right_new.clone()));
        assert_eq!(first[0].uri, "file:///tmp/a.rs");
        assert_eq!(first[0].diagnostics, vec![right_new.clone()]);

        // An unchanged right-side snapshot is ignored in favor of file://.
        let second = collect_new_diagnostics(&mut tracker, &files(right_new));
        assert_eq!(second[0].diagnostics, vec![disk_new.clone()]);

        // A newly changed right-side snapshot takes priority again.
        let third = collect_new_diagnostics(&mut tracker, &files(right_changed.clone()));
        assert_eq!(third[0].diagnostics, vec![right_changed]);
    }

    #[test]
    fn right_only_diagnostics_are_not_emitted_without_file_uri_peer() {
        let mut tracker = DiagnosticTrackerState::default();
        tracker
            .baseline
            .insert("/tmp/a.rs".to_string(), vec![diag("old", "Warning", 0)]);
        let files = vec![DiagnosticFile {
            uri: "_claude_fs_right:/tmp/a.rs".to_string(),
            diagnostics: vec![diag("new", "Error", 1)],
        }];
        assert!(collect_new_diagnostics(&mut tracker, &files).is_empty());
    }

    #[test]
    fn diagnostic_summary_matches_official_shape() {
        let files = vec![DiagnosticFile {
            uri: "/tmp/project/src/main.rs".to_string(),
            diagnostics: vec![diag("cannot find value", "Error", 4)],
        }];

        let summary = format_diagnostics_summary(&files);
        assert!(summary.starts_with("main.rs:\n"));
        assert!(summary.contains("[Line 5:3] cannot find value [E0425] (rust-analyzer)"));
    }

    #[test]
    fn diagnostic_display_summary_pluralizes_like_official() {
        let one = vec![DiagnosticFile {
            uri: "/a.rs".to_string(),
            diagnostics: vec![diag("broken", "Warning", 0)],
        }];
        assert_eq!(
            format_diagnostics_display_summary(&one),
            "Found 1 new diagnostic issue in 1 file"
        );

        let many = vec![
            DiagnosticFile {
                uri: "/a.rs".to_string(),
                diagnostics: vec![diag("a", "Error", 0), diag("b", "Hint", 1)],
            },
            DiagnosticFile {
                uri: "/b.rs".to_string(),
                diagnostics: vec![diag("c", "Info", 2)],
            },
        ];
        assert_eq!(
            format_diagnostics_display_summary(&many),
            "Found 3 new diagnostic issues in 2 files"
        );
    }

    #[test]
    fn diagnostic_summary_truncates_to_official_budget() {
        let files = vec![DiagnosticFile {
            uri: "/tmp/huge.rs".to_string(),
            diagnostics: vec![diag(&"x".repeat(5000), "Error", 0)],
        }];

        let summary = format_diagnostics_summary(&files);
        assert!(summary.ends_with(TRUNCATION_MARKER));
        assert_eq!(summary.chars().count(), MAX_DIAGNOSTICS_SUMMARY_CHARS);
    }
    #[tokio::test]
    async fn hook_query_preserves_parent_diagnostics_matches_official_generator_entry() {
        // execAgentHook.ts:177-189 calls query directly; only REPL.tsx:3565-3570
        // invokes handleQueryStart. Retain baseline so the next diagnostic is emitted.
        #[derive(Clone)]
        struct FailedModel;
        impl crate::query::deps::QueryDeps for FailedModel {}
        let old = diag("old", "Warning", 0);
        TRACKER
            .lock()
            .unwrap()
            .baseline
            .insert("/tmp/hook-parent.rs".into(), vec![old.clone()]);
        let handle = crate::query::spawn_query_generator(
            crate::query::QueryParams {
                turn_id: "hook-diagnostics".into(),
                input: "verify".into(),
                messages: Vec::new(),
                model_messages: vec![crate::types::message::Message::User(
                    crate::utils::messages::create_user_message("verify".into()),
                )],
                system_prompt: Vec::new(),
                user_context: Default::default(),
                system_context: Default::default(),
                query_source: crate::constants::query_source::QuerySource::HookAgent,
                token_budget: None,
                task_budget: None,
                max_turns: None,
                tool_use_context: crate::tool::ToolUseContext {
                    agent_id: Some("hook-agent-diagnostics".into()),
                    ..Default::default()
                },
            },
            FailedModel,
        );
        let control = handle.resume.as_ref().unwrap().clone();
        let _guard = crate::query::QueryGeneratorGuard(control.clone());
        while let Ok(event) = handle.events.recv().await {
            if matches!(event, crate::query::QueryEvent::Terminal(_)) {
                break;
            }
            control.advance();
        }
        let new = diag("new", "Error", 1);
        let result = collect_new_diagnostics(
            &mut TRACKER.lock().unwrap(),
            &[DiagnosticFile {
                uri: "file:///tmp/hook-parent.rs".into(),
                diagnostics: vec![old, new.clone()],
            }],
        );
        assert_eq!(
            result.len(),
            1,
            "hook query must not clear the parent's baseline"
        );
        assert_eq!(result[0].diagnostics, vec![new]);
    }
}
