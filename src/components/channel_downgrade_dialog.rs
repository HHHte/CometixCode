//! Maps to: CC `components/ChannelDowngradeDialog.tsx`.
//!
//! The official component owns only the dialog body and select options. Parent
//! components own the selected choice callback. Cometix mirrors that boundary:
//! this component renders the official dialog and exposes pure option helpers;
//! Settings keeps its existing keyboard state and applies the preview choice.

use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelDowngradeChoice {
    Downgrade,
    Stay,
    Cancel,
}

impl ChannelDowngradeChoice {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Downgrade => "downgrade",
            Self::Stay => "stay",
            Self::Cancel => "cancel",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "downgrade" => Some(Self::Downgrade),
            "stay" => Some(Self::Stay),
            "cancel" => Some(Self::Cancel),
            _ => None,
        }
    }
}

#[derive(Default, Props)]
pub struct ChannelDowngradeDialogProps {
    pub current_version: String,
    pub focused_index: usize,
}

pub fn channel_downgrade_options(current_version: &str) -> Vec<SelectOptionData> {
    vec![
        SelectOptionData {
            label: "Allow possible downgrade to stable version".to_string(),
            value: ChannelDowngradeChoice::Downgrade.as_str().to_string(),
            ..SelectOptionData::default()
        },
        SelectOptionData {
            label: format!("Stay on current version ({current_version}) until stable catches up"),
            value: ChannelDowngradeChoice::Stay.as_str().to_string(),
            ..SelectOptionData::default()
        },
    ]
}

#[component]
pub fn ChannelDowngradeDialog(
    props: &ChannelDowngradeDialogProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let options = channel_downgrade_options(&props.current_version);
    let focused_index = props.focused_index.min(options.len().saturating_sub(1));

    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(content: "Switch to Stable Channel".to_string(), color: theme.permission, weight: Weight::Bold)
            View(margin_top: 1u32) {
                Text(
                    content: format!(
                        "The stable channel may have an older version than what you're currently running ({}).",
                        props.current_version
                    ),
                    wrap: TextWrap::Wrap,
                )
            }
            Text(content: "How would you like to handle this?".to_string(), color: theme.inactive)
            View(margin_top: 1u32) {
                Select(
                    is_disabled: false,
                    hide_indexes: false,
                    visible_option_count: options.len(),
                    options: options,
                    focused_index: focused_index,
                    visible_from_index: 0usize,
                    layout: SelectLayout::Expanded,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn channel_downgrade_choices_match_official_values() {
        assert_eq!(ChannelDowngradeChoice::Downgrade.as_str(), "downgrade");
        assert_eq!(ChannelDowngradeChoice::Stay.as_str(), "stay");
        assert_eq!(ChannelDowngradeChoice::Cancel.as_str(), "cancel");
        assert_eq!(
            ChannelDowngradeChoice::from_value("downgrade"),
            Some(ChannelDowngradeChoice::Downgrade)
        );
        assert_eq!(ChannelDowngradeChoice::from_value("other"), None);
    }

    #[test]
    fn channel_downgrade_options_match_official_labels() {
        let options = channel_downgrade_options("1.2.3");
        assert_eq!(options.len(), 2);
        assert_eq!(
            options[0].label,
            "Allow possible downgrade to stable version"
        );
        assert_eq!(options[0].value, "downgrade");
        assert_eq!(
            options[1].label,
            "Stay on current version (1.2.3) until stable catches up"
        );
        assert_eq!(options[1].value, "stay");
    }

    #[test]
    fn channel_downgrade_dialog_renders_official_body_without_input_guide() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ChannelDowngradeDialog(current_version: "1.2.3".to_string(), focused_index: 0usize)
            }
        }
        .render(Some(120));
        let text = canvas.to_string();

        assert!(text.contains("Switch to Stable Channel"), "canvas=\n{text}");
        assert!(
            text.contains("The stable channel may have an older version than what you're currently running (1.2.3)."),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("How would you like to handle this?"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("Allow possible downgrade to stable version"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("Stay on current version (1.2.3) until stable catches up"),
            "canvas=\n{text}"
        );
        assert!(
            !text.contains("Enter to confirm"),
            "official ChannelDowngradeDialog hideInputGuide omits a separate footer; canvas=\n{text}"
        );
    }
}
