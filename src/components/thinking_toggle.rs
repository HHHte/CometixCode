//! Maps to: CC `components/ThinkingToggle.tsx`.
//!
//! Official React state (`confirmationPending`) and keybinding callbacks are
//! kept as caller-owned props in Cometix's render boundary. This preserves the
//! component/file split without moving readline or confirmation business logic
//! out of the existing Rust input flow.

use crate::components::configurable_shortcut_hint::ConfigurableShortcutHint;
use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use crate::components::design_system::byline::Byline;
use crate::components::design_system::keyboard_shortcut_hint::{
    KeyboardShortcutHint, KeyboardShortcutHintStyleContext,
};
use crate::components::design_system::pane::Pane;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ThinkingToggleProps {
    pub current_value: bool,
    pub focused_index: usize,
    pub confirmation_pending: Option<bool>,
    pub is_mid_conversation: bool,
    pub exit_pending: bool,
    pub exit_key_name: Option<String>,
}

/// Maps to: CC `components/ThinkingToggle.tsx` `options`.
///
/// Official `dimDescription` defaults to dimmed (`dimDescription !== false`);
/// omit explicit `false` so Compact two-column descriptions stay dim.
pub fn thinking_toggle_options() -> Vec<SelectOptionData> {
    vec![
        SelectOptionData {
            value: "true".to_string(),
            label: "Enabled".to_string(),
            description: Some("Claude will think before responding".to_string()),
            dim_description: true,
            ..SelectOptionData::default()
        },
        SelectOptionData {
            value: "false".to_string(),
            label: "Disabled".to_string(),
            description: Some("Claude will respond without extended thinking".to_string()),
            dim_description: true,
            ..SelectOptionData::default()
        },
    ]
}

/// Maps to: CC `components/ThinkingToggle.tsx#handleSelectChange` branch.
pub fn thinking_toggle_requires_confirmation(
    current_value: bool,
    selected: bool,
    is_mid_conversation: bool,
) -> bool {
    is_mid_conversation && selected != current_value
}

#[component]
pub fn ThinkingToggle(props: &ThinkingToggleProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let selected_value = if props.current_value { "true" } else { "false" }.to_string();
    let options = thinking_toggle_options();
    let focused_index = props.focused_index.min(options.len().saturating_sub(1));
    let exit_key_name = props
        .exit_key_name
        .as_deref()
        .filter(|value| !value.is_empty())
        .unwrap_or("Ctrl-C")
        .to_string();
    let confirmation_pending = props.confirmation_pending;
    let _ = props.is_mid_conversation;

    // Maps to CC footer: `<Text dimColor italic>{…Byline…}</Text>`.
    let footer_style = KeyboardShortcutHintStyleContext {
        dim: true,
        italic: true,
    };

    element! {
        Pane(color: Some(theme.permission)) {
            View(flex_direction: FlexDirection::Column) {
                View(flex_direction: FlexDirection::Column, margin_bottom: 1u32) {
                    Text(content: "Toggle thinking mode".to_string(), color: theme.remember, weight: Weight::Bold)
                    Text(content: "Enable or disable thinking for this session.".to_string(), dim: true)
                }
                #(if confirmation_pending.is_some() {
                    element! {
                        View(flex_direction: FlexDirection::Column, margin_bottom: 1u32, row_gap: 1u32) {
                            Text(content: "Changing thinking mode mid-conversation will increase latency and may reduce quality. For best results, set this at the start of a session.".to_string(), color: theme.warning)
                            Text(content: "Do you want to proceed?".to_string(), color: theme.warning)
                        }
                    }.into_any()
                } else {
                    // CC ThinkingToggle Select omits `layout` → default
                    // `compact`, which with descriptions uses the two-column
                    // label | description row (not Expanded stacked rows).
                    element! {
                        View(flex_direction: FlexDirection::Column, margin_bottom: 1u32) {
                            Select(
                                options: options,
                                focused_index: focused_index,
                                selected_value: Some(selected_value),
                                visible_option_count: 2usize,
                                visible_from_index: 0usize,
                                layout: SelectLayout::Compact,
                            )
                        }
                    }.into_any()
                })
            }
            ContextProvider(value: Context::owned(footer_style)) {
                #(if props.exit_pending {
                    element! {
                        Text(
                            content: format!("Press {exit_key_name} again to exit"),
                            dim: true,
                            italic: true,
                        )
                    }.into_any()
                } else if confirmation_pending.is_some() {
                    element! {
                        Byline {
                            KeyboardShortcutHint(shortcut: "Enter".to_string(), action: "confirm".to_string())
                            ConfigurableShortcutHint(
                                action: "confirm:no".to_string(),
                                context: "Confirmation".to_string(),
                                fallback: "Esc".to_string(),
                                description: "cancel".to_string(),
                            )
                        }
                    }.into_any()
                } else {
                    element! {
                        Byline {
                            KeyboardShortcutHint(shortcut: "Enter".to_string(), action: "confirm".to_string())
                            ConfigurableShortcutHint(
                                action: "confirm:no".to_string(),
                                context: "Confirmation".to_string(),
                                fallback: "Esc".to_string(),
                                description: "exit".to_string(),
                            )
                        }
                    }.into_any()
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render(props: ThinkingToggleProps) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ThinkingToggle(
                    current_value: props.current_value,
                    focused_index: props.focused_index,
                    confirmation_pending: props.confirmation_pending,
                    is_mid_conversation: props.is_mid_conversation,
                    exit_pending: props.exit_pending,
                    exit_key_name: props.exit_key_name,
                )
            }
        }
        .render(Some(100))
        .to_string()
    }

    #[test]
    fn thinking_toggle_options_match_official_copy() {
        let options = thinking_toggle_options();
        assert_eq!(options[0].value, "true");
        assert_eq!(options[0].label, "Enabled");
        assert_eq!(
            options[0].description.as_deref(),
            Some("Claude will think before responding")
        );
        assert_eq!(options[1].value, "false");
        assert_eq!(options[1].label, "Disabled");
    }

    #[test]
    fn thinking_toggle_confirmation_gate_matches_official_branch() {
        assert!(thinking_toggle_requires_confirmation(true, false, true));
        assert!(!thinking_toggle_requires_confirmation(true, true, true));
        assert!(!thinking_toggle_requires_confirmation(true, false, false));
    }

    #[test]
    fn thinking_toggle_renders_select_and_exit_footer() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ThinkingToggle(
                    current_value: true,
                    focused_index: 0usize,
                    confirmation_pending: None,
                    is_mid_conversation: false,
                    exit_pending: false,
                    exit_key_name: None,
                )
            }
        }
        .render(Some(100));
        let text = canvas.to_string();
        assert!(text.contains("Toggle thinking mode"), "canvas=\n{text}");
        // Compact default: numbered indexes + description on the same row family.
        assert!(text.contains("1. Enabled"), "canvas=\n{text}");
        assert!(text.contains("2. Disabled"), "canvas=\n{text}");
        assert!(
            text.contains("Claude will think before responding"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("Enter to confirm · Esc to exit"),
            "canvas=\n{text}"
        );
        // Footer inherits CC `<Text dimColor italic>`.
        let footer_line = text
            .lines()
            .find(|line| line.contains("Enter to confirm"))
            .expect("footer line");
        let enter_col = footer_line.find("Enter to confirm").unwrap();
        let footer_y = text
            .lines()
            .position(|line| line.contains("Enter to confirm"))
            .unwrap();
        let style = canvas
            .resolved_text_style(enter_col, footer_y)
            .expect("footer style");
        assert!(
            style.italic,
            "expected italic footer like official Text dimColor italic; canvas=\n{text}"
        );
        assert!(style.dim, "expected dim footer; canvas=\n{text}");
    }

    #[test]
    fn thinking_toggle_renders_mid_conversation_confirmation() {
        let text = render(ThinkingToggleProps {
            current_value: true,
            confirmation_pending: Some(false),
            is_mid_conversation: true,
            ..Default::default()
        });
        assert!(
            text.contains("Changing thinking mode mid-conversation"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Do you want to proceed?"), "canvas=\n{text}");
        assert!(
            text.contains("Enter to confirm · Esc to cancel"),
            "canvas=\n{text}"
        );
    }

    #[test]
    fn thinking_toggle_renders_double_ctrl_c_exit_state() {
        let text = render(ThinkingToggleProps {
            exit_pending: true,
            exit_key_name: Some("Ctrl-C".to_string()),
            ..Default::default()
        });
        assert!(
            text.contains("Press Ctrl-C again to exit"),
            "canvas=\n{text}"
        );
    }
}
