//! Display helpers for CC `tools/ExitWorktreeTool/UI.tsx`.

pub fn render_tool_use_message() -> &'static str {
    "Exiting worktree…"
}

use super::Output;
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `action` must be the keep/remove enum,
/// `originalCwd`/`worktreePath`/`message` are required
/// (`ExitWorktreeTool.ts:47-58`).
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    let optional_string = |key: &str| -> Option<Option<String>> {
        match map.get(key) {
            None => Some(None),
            Some(serde_json::Value::String(value)) => Some(Some(value.clone())),
            Some(_) => None,
        }
    };
    let optional_number = |key: &str| -> Option<Option<u64>> {
        match map.get(key) {
            None => Some(None),
            Some(serde_json::Value::Number(value)) => Some(Some(value.as_u64()?)),
            Some(_) => None,
        }
    };
    let action = map.get("action")?.as_str()?.to_string();
    if !matches!(action.as_str(), "keep" | "remove") {
        return None;
    }
    Some(Output {
        action,
        original_cwd: map.get("originalCwd")?.as_str()?.to_string(),
        worktree_path: map.get("worktreePath")?.as_str()?.to_string(),
        worktree_branch: optional_string("worktreeBranch")?,
        tmux_session_name: optional_string("tmuxSessionName")?,
        discarded_files: optional_number("discardedFiles")?,
        discarded_commits: optional_number("discardedCommits")?,
        message: map.get("message")?.as_str()?.to_string(),
    })
}

/// Serializes [`Output`] to CC's exact `toolUseResult` wire shape —
/// optional fields absent, never null.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "action".to_string(),
        serde_json::Value::String(output.action.clone()),
    );
    map.insert(
        "originalCwd".to_string(),
        serde_json::Value::String(output.original_cwd.clone()),
    );
    map.insert(
        "worktreePath".to_string(),
        serde_json::Value::String(output.worktree_path.clone()),
    );
    if let Some(branch) = output.worktree_branch.as_ref() {
        map.insert(
            "worktreeBranch".to_string(),
            serde_json::Value::String(branch.clone()),
        );
    }
    if let Some(session) = output.tmux_session_name.as_ref() {
        map.insert(
            "tmuxSessionName".to_string(),
            serde_json::Value::String(session.clone()),
        );
    }
    if let Some(files) = output.discarded_files {
        map.insert("discardedFiles".to_string(), serde_json::json!(files));
    }
    if let Some(commits) = output.discarded_commits {
        map.insert("discardedCommits".to_string(), serde_json::json!(commits));
    }
    map.insert(
        "message".to_string(),
        serde_json::Value::String(output.message.clone()),
    );
    serde_json::Value::Object(map)
}

/// Maps to: CC `ExitWorktreeTool/UI.tsx:12-33` `renderToolResultMessage` —
/// "Kept/Removed worktree" plus the optional branch tag and the dim
/// "Returned to {originalCwd}" line.
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    let action_label = if output.action == "keep" {
        "Kept worktree"
    } else {
        "Removed worktree"
    };
    vec![
        ToolRenderLine::new(
            output
                .worktree_branch
                .as_ref()
                .map(|branch| format!("{action_label} (branch {branch})"))
                .unwrap_or_else(|| action_label.to_string()),
            ToolRenderTone::Normal,
        ),
        ToolRenderLine::new(
            format!("Returned to {}", output.original_cwd),
            ToolRenderTone::Inactive,
        ),
    ]
}
