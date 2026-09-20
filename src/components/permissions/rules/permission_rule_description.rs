//! Maps to: CC `components/permissions/rules/PermissionRuleDescription.tsx`.

use crate::types::permissions::PermissionRuleValue;
use iocraft::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionRuleDescriptionKind {
    BashPrefix(String),
    BashExact(String),
    AnyBashCommand,
    AnyToolUse(String),
    NoDescription,
}

/// Maps to: CC `PermissionRuleDescription(...)` switch on `ruleValue.toolName`.
pub fn permission_rule_description_kind(
    rule_value: &PermissionRuleValue,
) -> PermissionRuleDescriptionKind {
    if rule_value.tool_name == crate::tools::bash_tool::tool_name::BASH_TOOL_NAME {
        match rule_value
            .rule_content
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            Some(content) if content.ends_with(":*") => {
                PermissionRuleDescriptionKind::BashPrefix(content[..content.len() - 2].to_string())
            }
            Some(content) => PermissionRuleDescriptionKind::BashExact(content.to_string()),
            None => PermissionRuleDescriptionKind::AnyBashCommand,
        }
    } else if rule_value.rule_content.as_deref().is_none_or(str::is_empty) {
        PermissionRuleDescriptionKind::AnyToolUse(rule_value.tool_name.clone())
    } else {
        PermissionRuleDescriptionKind::NoDescription
    }
}

pub fn permission_rule_description_text(rule_value: &PermissionRuleValue) -> Option<String> {
    match permission_rule_description_kind(rule_value) {
        PermissionRuleDescriptionKind::BashPrefix(prefix) => {
            Some(format!("Any Bash command starting with {prefix}"))
        }
        PermissionRuleDescriptionKind::BashExact(command) => {
            Some(format!("The Bash command {command}"))
        }
        PermissionRuleDescriptionKind::AnyBashCommand => Some("Any Bash command".to_string()),
        PermissionRuleDescriptionKind::AnyToolUse(tool_name) => {
            Some(format!("Any use of the {tool_name} tool"))
        }
        PermissionRuleDescriptionKind::NoDescription => None,
    }
}

#[derive(Default, Props)]
pub struct PermissionRuleDescriptionProps {
    pub rule_value: Option<PermissionRuleValue>,
}

/// Maps to: CC `PermissionRuleDescription` render path.
#[component]
pub fn PermissionRuleDescription(
    props: &PermissionRuleDescriptionProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let _ = hooks;
    let Some(rule) = props.rule_value.as_ref() else {
        return element! { View }.into_any();
    };
    let (prefix, emphasized, suffix) = match permission_rule_description_kind(rule) {
        PermissionRuleDescriptionKind::BashPrefix(value) => {
            ("Any Bash command starting with ", Some(value), "")
        }
        PermissionRuleDescriptionKind::BashExact(value) => ("The Bash command ", Some(value), ""),
        PermissionRuleDescriptionKind::AnyBashCommand => ("Any Bash command", None, ""),
        PermissionRuleDescriptionKind::AnyToolUse(value) => {
            ("Any use of the ", Some(value), " tool")
        }
        PermissionRuleDescriptionKind::NoDescription => return element! { View }.into_any(),
    };
    // L1 Explicit structured Ink text-flow carrier: nested bold inherits dim.
    let mut segments = vec![StyledSegment::new(prefix)];
    if let Some(value) = emphasized {
        let mut part = StyledSegment::new(value);
        part.styles.bold = Some(true);
        segments.push(part);
    }
    segments.push(StyledSegment::new(suffix));
    element! { Text(segments: Some(segments), dim: true, wrap: TextWrap::Wrap) }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn permission_rule_description_matches_official_bash_and_tool_cases() {
        assert_eq!(
            permission_rule_description_text(&PermissionRuleValue::new(
                "Bash",
                Some("cargo test:*".to_string())
            ))
            .as_deref(),
            Some("Any Bash command starting with cargo test")
        );
        assert_eq!(
            permission_rule_description_text(&PermissionRuleValue::new(
                "Bash",
                Some("rm -rf /tmp/x".to_string())
            ))
            .as_deref(),
            Some("The Bash command rm -rf /tmp/x")
        );
        assert_eq!(
            permission_rule_description_text(&PermissionRuleValue::new("WebFetch", None))
                .as_deref(),
            Some("Any use of the WebFetch tool")
        );
    }

    #[test]
    fn permission_rule_description_renders_dim_text() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                PermissionRuleDescription(
                    rule_value: Some(PermissionRuleValue::new("Bash", Some("ls:*".to_string())))
                )
            }
        }
        .render(Some(80))
        .to_string();
        assert!(
            text.contains("Any Bash command starting with ls"),
            "canvas=\n{text}"
        );
    }

    #[test]
    fn permission_rule_description_matches_official_single_suffix_and_inherited_dim_bold() {
        let rule = PermissionRuleValue::new("Bash", Some("echo:*:*".to_string()));
        assert_eq!(
            permission_rule_description_text(&rule).as_deref(),
            Some("Any Bash command starting with echo:*")
        );
        let canvas =
            element! { PermissionRuleDescription(rule_value: Some(rule)) }.render(Some(80));
        let text = canvas.to_string();
        let x = text.find("echo:*").unwrap();
        let emphasized = canvas.resolved_text_style(x, 0).unwrap();
        assert!(emphasized.dim);
        assert_eq!(emphasized.weight, Weight::Bold);
        let prefix = canvas.resolved_text_style(0, 0).unwrap();
        assert!(prefix.dim);
        assert_ne!(prefix.weight, Weight::Bold);
    }
}
