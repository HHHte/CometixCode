//! Display helpers for CC `tools/EnterWorktreeTool/UI.tsx`.

pub fn render_tool_use_message() -> &'static str {
    "Creating worktree…"
}

use super::Output;
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `worktreePath` and `message` are
/// required, `worktreeBranch` optional (`EnterWorktreeTool.ts:42-48`).
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    let worktree_branch = match map.get("worktreeBranch") {
        None => None,
        Some(serde_json::Value::String(branch)) => Some(branch.clone()),
        Some(_) => return None,
    };
    Some(Output {
        worktree_path: map.get("worktreePath")?.as_str()?.to_string(),
        worktree_branch,
        message: map.get("message")?.as_str()?.to_string(),
    })
}

/// Serializes [`Output`] to CC's exact `toolUseResult` wire shape.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    let mut map = serde_json::Map::new();
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
    map.insert(
        "message".to_string(),
        serde_json::Value::String(output.message.clone()),
    );
    serde_json::Value::Object(map)
}

/// Maps to: CC `EnterWorktreeTool/UI.tsx:12-25` `renderToolResultMessage` —
/// "Switched to worktree on branch {branch}" plus the dim path. An absent
/// branch renders empty inline, exactly as the JSX interpolates `undefined`.
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    vec![
        ToolRenderLine::new(
            format!(
                "Switched to worktree on branch {}",
                output.worktree_branch.as_deref().unwrap_or_default()
            ),
            ToolRenderTone::Normal,
        ),
        ToolRenderLine::new(output.worktree_path, ToolRenderTone::Inactive),
    ]
}
