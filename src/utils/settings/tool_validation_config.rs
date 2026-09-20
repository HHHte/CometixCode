//! Tool validation configuration.
//!
//! Maps to: CC `utils/settings/toolValidationConfig.ts`.
//!
//! Most tools need NO configuration — basic validation works automatically.
//! Only add a tool here if it has special pattern requirements.

use super::permission_validation::PermissionRuleValidation;

/// Tools that accept file glob patterns (e.g., *.ts, src/**).
const FILE_PATTERN_TOOLS: &[&str] = &[
    "Read",
    "Write",
    "Edit",
    "Glob",
    "NotebookRead",
    "NotebookEdit",
];

/// Tools that accept bash wildcard patterns (* anywhere) and legacy :* prefix
/// syntax.
const BASH_PREFIX_TOOLS: &[&str] = &["Bash"];

/// Maps to: CC `isFilePatternTool(toolName)`.
pub fn is_file_pattern_tool(tool_name: &str) -> bool {
    FILE_PATTERN_TOOLS.contains(&tool_name)
}

/// Maps to: CC `isBashPrefixTool(toolName)`.
pub fn is_bash_prefix_tool(tool_name: &str) -> bool {
    BASH_PREFIX_TOOLS.contains(&tool_name)
}

/// Maps to: CC `getCustomValidation(toolName)` — the `customValidation` map of
/// `TOOL_VALIDATION_CONFIG`.
pub fn get_custom_validation(tool_name: &str) -> Option<fn(&str) -> PermissionRuleValidation> {
    match tool_name {
        "WebSearch" => Some(validate_web_search),
        "WebFetch" => Some(validate_web_fetch),
        _ => None,
    }
}

/// WebSearch doesn't support wildcards or complex patterns.
fn validate_web_search(content: &str) -> PermissionRuleValidation {
    if content.contains('*') || content.contains('?') {
        return PermissionRuleValidation {
            valid: false,
            error: Some("WebSearch does not support wildcards".to_string()),
            suggestion: Some("Use exact search terms without * or ?".to_string()),
            examples: vec![
                "WebSearch(claude ai)".to_string(),
                "WebSearch(typescript tutorial)".to_string(),
            ],
        };
    }
    PermissionRuleValidation::valid()
}

/// WebFetch uses domain: prefix for hostname-based permissions.
fn validate_web_fetch(content: &str) -> PermissionRuleValidation {
    // Check if it's trying to use a URL format.
    if content.contains("://") || content.starts_with("http") {
        return PermissionRuleValidation {
            valid: false,
            error: Some("WebFetch permissions use domain format, not URLs".to_string()),
            suggestion: Some("Use \"domain:hostname\" format".to_string()),
            examples: vec![
                "WebFetch(domain:example.com)".to_string(),
                "WebFetch(domain:github.com)".to_string(),
            ],
        };
    }

    // Must start with domain: prefix.
    if !content.starts_with("domain:") {
        return PermissionRuleValidation {
            valid: false,
            error: Some("WebFetch permissions must use \"domain:\" prefix".to_string()),
            suggestion: Some("Use \"domain:hostname\" format".to_string()),
            examples: vec![
                "WebFetch(domain:example.com)".to_string(),
                "WebFetch(domain:*.google.com)".to_string(),
            ],
        };
    }

    // Wildcards are allowed in domain patterns (domain:*.example.com, ...).
    PermissionRuleValidation::valid()
}
