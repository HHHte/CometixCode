//! Permission rule string parser.
//! Maps to: CC `utils/permissions/permissionRuleParser.ts`.

use crate::types::permissions::PermissionRuleValue;

/// Maps legacy tool names to current canonical names.
/// Maps to CC `LEGACY_TOOL_NAME_ALIASES`.
pub fn normalize_legacy_tool_name(name: &str) -> String {
    match name {
        "Task" => crate::tools::agent_tool::constants::AGENT_TOOL_NAME.to_string(),
        "KillShell" => crate::tools::task_stop_tool::prompt::TASK_STOP_TOOL_NAME.to_string(),
        "AgentOutputTool" | "BashOutputTool" => {
            crate::tools::task_output_tool::constants::TASK_OUTPUT_TOOL_NAME.to_string()
        }
        "Brief" => "Brief".to_string(),
        _ => name.to_string(),
    }
}

pub fn get_legacy_tool_names(canonical_name: &str) -> Vec<String> {
    [
        ("Task", crate::tools::agent_tool::constants::AGENT_TOOL_NAME),
        (
            "KillShell",
            crate::tools::task_stop_tool::prompt::TASK_STOP_TOOL_NAME,
        ),
        (
            "AgentOutputTool",
            crate::tools::task_output_tool::constants::TASK_OUTPUT_TOOL_NAME,
        ),
        (
            "BashOutputTool",
            crate::tools::task_output_tool::constants::TASK_OUTPUT_TOOL_NAME,
        ),
        ("Brief", "Brief"),
    ]
    .into_iter()
    .filter_map(|(legacy, canonical)| (canonical == canonical_name).then_some(legacy.to_string()))
    .collect()
}

/// Maps to CC `escapeRuleContent(...)`.
pub fn escape_rule_content(content: &str) -> String {
    content
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// Maps to CC `unescapeRuleContent(...)`.
pub fn unescape_rule_content(content: &str) -> String {
    content
        .replace("\\(", "(")
        .replace("\\)", ")")
        .replace("\\\\", "\\")
}

/// Maps to CC `permissionRuleValueFromString(...)`.
pub fn permission_rule_value_from_string(rule_string: &str) -> PermissionRuleValue {
    let open_paren_index = find_first_unescaped_char(rule_string, '(');
    let Some(open_paren_index) = open_paren_index else {
        return PermissionRuleValue::new(normalize_legacy_tool_name(rule_string), None);
    };

    let Some(close_paren_index) = find_last_unescaped_char(rule_string, ')') else {
        return PermissionRuleValue::new(normalize_legacy_tool_name(rule_string), None);
    };
    if close_paren_index <= open_paren_index || close_paren_index != rule_string.len() - 1 {
        return PermissionRuleValue::new(normalize_legacy_tool_name(rule_string), None);
    }

    let tool_name = &rule_string[..open_paren_index];
    if tool_name.is_empty() {
        return PermissionRuleValue::new(normalize_legacy_tool_name(rule_string), None);
    }

    let raw_content = &rule_string[open_paren_index + 1..close_paren_index];
    if raw_content.is_empty() || raw_content == "*" {
        return PermissionRuleValue::new(normalize_legacy_tool_name(tool_name), None);
    }

    PermissionRuleValue::new(
        normalize_legacy_tool_name(tool_name),
        Some(unescape_rule_content(raw_content)),
    )
}

/// Maps to CC `permissionRuleValueToString(...)`.
pub fn permission_rule_value_to_string(rule_value: &PermissionRuleValue) -> String {
    match rule_value.rule_content.as_deref() {
        Some(content) if !content.is_empty() => {
            format!("{}({})", rule_value.tool_name, escape_rule_content(content))
        }
        _ => rule_value.tool_name.clone(),
    }
}

fn find_first_unescaped_char(value: &str, target: char) -> Option<usize> {
    for (index, ch) in value.char_indices() {
        if ch == target && !is_escaped_at(value.as_bytes(), index) {
            return Some(index);
        }
    }
    None
}

fn find_last_unescaped_char(value: &str, target: char) -> Option<usize> {
    for (index, ch) in value.char_indices().rev() {
        if ch == target && !is_escaped_at(value.as_bytes(), index) {
            return Some(index);
        }
    }
    None
}

fn is_escaped_at(bytes: &[u8], index: usize) -> bool {
    let mut backslash_count = 0usize;
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslash_count += 1;
        cursor -= 1;
    }
    backslash_count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_escaped_permission_rule_values_like_official() {
        let parsed = permission_rule_value_from_string(r"Bash(psycopg2.connect\(\))");
        assert_eq!(parsed.tool_name, "Bash");
        assert_eq!(parsed.rule_content.as_deref(), Some("psycopg2.connect()"));
        assert_eq!(
            permission_rule_value_to_string(&parsed),
            r"Bash(psycopg2.connect\(\))"
        );
    }

    #[test]
    fn parses_tool_wide_and_legacy_rule_names_like_official() {
        assert_eq!(
            permission_rule_value_from_string("Bash(*)"),
            PermissionRuleValue::new("Bash", None)
        );
        assert_eq!(
            permission_rule_value_from_string("Task").tool_name,
            crate::tools::agent_tool::constants::AGENT_TOOL_NAME
        );
    }
}
