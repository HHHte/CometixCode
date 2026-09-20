//! Maps to: CC `components/sandbox/SandboxOverridesTab.tsx`.

use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverrideMode {
    Open,
    Closed,
}

impl OverrideMode {
    pub fn value(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

pub fn sandbox_override_options(current_mode: OverrideMode) -> Vec<SelectOptionData> {
    [
        (OverrideMode::Open, "Allow unsandboxed fallback"),
        (OverrideMode::Closed, "Strict sandbox mode"),
    ]
    .into_iter()
    .map(|(mode, label)| SelectOptionData {
        label: if mode == current_mode {
            format!("{label} (current)")
        } else {
            label.to_string()
        },
        description: None,
        dim_description: true,
        value: mode.value().to_string(),
        disabled: false,
        input: None,
    })
    .collect()
}

#[derive(Default, Props)]
pub struct SandboxOverridesTabProps {
    pub is_enabled: bool,
    pub is_locked: bool,
    pub current_allow_unsandboxed: bool,
    pub focused_index: usize,
}

#[component]
pub fn SandboxOverridesTab(
    props: &SandboxOverridesTabProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();

    if !props.is_enabled {
        return element! {
            View(flex_direction: FlexDirection::Column, padding_top: 1u32, padding_bottom: 1u32) {
                Text(content: "Sandbox is not enabled. Enable sandbox to configure override settings.".to_string(), color: theme.inactive, wrap: TextWrap::Wrap)
            }
        }
        .into_any();
    }

    if props.is_locked {
        return element! {
            View(flex_direction: FlexDirection::Column, padding_top: 1u32, padding_bottom: 1u32) {
                Text(content: "Override settings are managed by a higher-priority configuration and cannot be changed locally.".to_string(), color: theme.inactive, wrap: TextWrap::Wrap)
                View(margin_top: 1u32) {
                    Text(content: format!("Current setting: {}", if props.current_allow_unsandboxed { "Allow unsandboxed fallback" } else { "Strict sandbox mode" }), color: theme.inactive, wrap: TextWrap::NoWrap)
                }
            }
        }
        .into_any();
    }

    let current_mode = if props.current_allow_unsandboxed {
        OverrideMode::Open
    } else {
        OverrideMode::Closed
    };

    element! {
        View(flex_direction: FlexDirection::Column, padding_top: 1u32, padding_bottom: 1u32) {
            View(margin_bottom: 1u32) {
                Text(content: "Configure Overrides:".to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
            }
            Select(
                options: sandbox_override_options(current_mode),
                focused_index: props.focused_index,
                visible_option_count: 2usize,
                layout: SelectLayout::Compact,
                hide_indexes: true,
            )
            View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
                Text(content: "Allow unsandboxed fallback: When a command fails due to sandbox restrictions, Claude can retry with dangerouslyDisableSandbox to run outside the sandbox (falling back to default permissions).".to_string(), color: theme.inactive, wrap: TextWrap::Wrap)
                Text(content: "Strict sandbox mode: All bash commands invoked by the model must run in the sandbox unless they are explicitly listed in excludedCommands.".to_string(), color: theme.inactive, wrap: TextWrap::Wrap)
                Text(content: "Learn more: https://code.claude.com/docs/en/sandboxing#configure-sandboxing".to_string(), color: theme.inactive, wrap: TextWrap::Wrap)
            }
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn sandbox_override_options_mark_current_like_official() {
        let options = sandbox_override_options(OverrideMode::Closed);
        assert_eq!(options[0].label, "Allow unsandboxed fallback");
        assert_eq!(options[1].label, "Strict sandbox mode (current)");
    }

    #[test]
    fn sandbox_overrides_disabled_branch_matches_official_copy() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                SandboxOverridesTab(is_enabled: false, is_locked: false, current_allow_unsandboxed: true)
            }
        }
        .render(Some(100))
        .to_string();
        assert!(text.contains("Sandbox is not enabled"), "canvas=\n{text}");
    }
}
