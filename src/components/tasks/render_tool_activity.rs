//! Maps to: CC `components/tasks/renderToolActivity.tsx:1-39`.

use crate::types::tools::{Tool, find_tool_by_name};

#[derive(Clone, Debug, PartialEq)]
pub struct ToolActivity {
    pub tool_name: String,
    pub input: serde_json::Value,
}

/// Typed UI callback projection for the behavioral fields (`userFacingName`,
/// `renderToolUseMessage`) that are not part of Rust's API schema metadata.
pub fn render_tool_activity(
    activity: &ToolActivity,
    tools: &[Tool],
    user_facing: impl Fn(&Tool, &serde_json::Value) -> Result<(Option<String>, Option<String>), String>,
) -> String {
    let Some(tool) = find_tool_by_name(tools, &activity.tool_name) else {
        return activity.tool_name.clone();
    };
    match user_facing(tool, &activity.input) {
        Ok((Some(name), Some(arguments))) if !arguments.is_empty() => {
            format!("{name}({arguments})")
        }
        Ok((Some(name), _)) if !name.is_empty() => name,
        _ => activity.tool_name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tool() -> Tool {
        Tool {
            name: "Read".to_string(),
            input_schema: serde_json::json!({"type":"object"}),
            ..Default::default()
        }
    }
    #[test]
    fn unknown_parse_failure_empty_name_and_arguments_follow_fallback_contract() {
        let activity = ToolActivity {
            tool_name: "Read".to_string(),
            input: serde_json::json!({"file_path":"a.rs"}),
        };
        assert_eq!(
            render_tool_activity(&activity, &[], |_, _| unreachable!()),
            "Read"
        );
        assert_eq!(
            render_tool_activity(&activity, &[tool()], |_, _| Err("bad".to_string())),
            "Read"
        );
        assert_eq!(
            render_tool_activity(&activity, &[tool()], |_, _| Ok((
                Some("Read".to_string()),
                Some("a.rs".to_string())
            ))),
            "Read(a.rs)"
        );
    }
}
