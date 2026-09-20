//! Maps to: CC `components/ApproveApiKey.tsx`:1-65.
//!
//! Official `ApproveApiKey` writes the user's approval/rejection into global
//! config before calling `onDone`. Cometix keeps this component render-only for
//! the current onboarding slice: callers own key handling and global-config
//! persistence remains separately gated from live session transcript writes.

use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use crate::components::design_system::dialog::Dialog;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::collections::BTreeMap;

pub(crate) fn approve_api_key_options() -> Vec<SelectOptionData> {
    vec![
        SelectOptionData {
            label: "Yes".to_string(),
            value: "yes".to_string(),
            ..SelectOptionData::default()
        },
        SelectOptionData {
            label: "No (recommended)".to_string(),
            value: "no".to_string(),
            ..SelectOptionData::default()
        },
    ]
}

#[derive(Default, Props)]
pub(crate) struct ApproveApiKeyProps {
    pub custom_api_key_truncated: String,
    pub focused_index: usize,
}

/// Maps to: CC `components/ApproveApiKey.tsx`:14-65 `ApproveApiKey(...)`.
#[component]
pub(crate) fn ApproveApiKey(
    props: &ApproveApiKeyProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = *hooks.use_context::<Theme>();
    let options = approve_api_key_options();
    let focused_index = props.focused_index.min(options.len().saturating_sub(1));
    let custom_api_key_truncated = props.custom_api_key_truncated.clone();
    // CC label is nested Text: only "recommended" is bold.
    let mut recommended = StyledSegment::new("recommended");
    recommended.styles.bold = Some(true);
    let option_labels = BTreeMap::from([(
        "no".to_string(),
        vec![
            StyledSegment::new("No ("),
            recommended,
            StyledSegment::new(")"),
        ],
    )]);

    element! {
        Dialog(
            title: "Detected a custom API key in your environment".to_string(),
            color: Some(theme.warning),
            on_cancel: |_| {},
        ) {
            View(flex_direction: FlexDirection::Row) {
                Text(content: "ANTHROPIC_API_KEY".to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
                Text(content: format!(": sk-ant-...{custom_api_key_truncated}"), wrap: TextWrap::NoWrap)
            }
            Text(content: "Do you want to use this API key?".to_string())
            Select(
                is_disabled: false,
                hide_indexes: false,
                visible_option_count: options.len(),
                options: options,
                option_labels,
                focused_index: focused_index,
                selected_value: Some("no".to_string()),
                visible_from_index: 0usize,
                layout: SelectLayout::Compact,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn approve_api_key_renders_official_dialog_copy_and_options() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ApproveApiKey(
                    custom_api_key_truncated: "abcd1234".to_string(),
                    focused_index: 1usize,
                )
            }
        }
        .render(Some(100))
        .to_string();

        assert!(
            text.contains("Detected a custom API key in your environment"),
            "canvas=\n{text}"
        );
        assert!(text.contains("ANTHROPIC_API_KEY"), "canvas=\n{text}");
        assert!(text.contains("sk-ant-...abcd1234"), "canvas=\n{text}");
        assert!(
            text.contains("Do you want to use this API key?"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Yes"), "canvas=\n{text}");
        assert!(text.contains("No (recommended)"), "canvas=\n{text}");
    }

    #[test]
    fn approve_api_key_select_preserves_source_child_bold_and_compact_layout() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ApproveApiKey(custom_api_key_truncated: "abcd".to_string(), focused_index: 1usize)
            }
        }
        .render(Some(100));
        let text = canvas.to_string();
        let lines = text.lines().collect::<Vec<_>>();
        let row = lines
            .iter()
            .position(|line| line.contains("No (recommended)"))
            .unwrap();
        assert!(lines[row].contains("2. No (recommended)"), "{text}");
        assert!(
            lines[row - 1].contains("1. Yes"),
            "source compact has no expanded blank row: {text}"
        );
        for (needle, expected) in [("No (", Weight::Normal), ("recommended", Weight::Bold)] {
            let offset = lines[row].find(needle).unwrap();
            let column = unicode_width::UnicodeWidthStr::width(&lines[row][..offset]);
            assert_eq!(
                canvas.resolved_text_style(column, row).unwrap().weight,
                expected,
                "{needle}: {text}"
            );
        }
    }
}
