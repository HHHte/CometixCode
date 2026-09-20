//! Maps to: CC `components/EffortCallout.tsx`.
//!
//! Persistence (`markV2Dismissed`, `updateSettingsForSource`) and subscriber
//! detection remain outside this render boundary. Pure decision helpers expose
//! the official branch behavior with caller-provided auth/config state.

use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use crate::components::effort_indicator::effort_level_to_symbol;
use crate::components::permissions::permission_dialog::PermissionDialog;
use crate::utils::effort::{OpusDefaultEffortConfig, get_opus_default_effort_config};
use iocraft::prelude::*;
use std::collections::BTreeMap;

pub const EFFORT_CALLOUT_AUTO_DISMISS_MS: u64 = 30_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffortCalloutSelection {
    Low,
    Medium,
    High,
    Dismiss,
}

impl EffortCalloutSelection {
    pub fn as_official_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Dismiss => "dismiss",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffortCalloutDecision {
    pub show: bool,
    pub mark_v2_dismissed: bool,
}

/// Maps to: CC `components/EffortCallout.tsx#shouldShowEffortCallout`.
pub fn should_show_effort_callout_with_state(
    model: &str,
    effort_callout_v2_dismissed: bool,
    num_startups: u64,
    is_pro_subscriber: bool,
    effort_callout_dismissed: bool,
    is_max_subscriber: bool,
    is_team_subscriber: bool,
    opus_config_enabled: bool,
) -> EffortCalloutDecision {
    if !model.to_lowercase().contains("opus-4-6") {
        return EffortCalloutDecision::default();
    }
    if effort_callout_v2_dismissed {
        return EffortCalloutDecision::default();
    }
    if num_startups <= 1 {
        return EffortCalloutDecision {
            show: false,
            mark_v2_dismissed: true,
        };
    }
    if is_pro_subscriber {
        if effort_callout_dismissed {
            return EffortCalloutDecision {
                show: false,
                mark_v2_dismissed: true,
            };
        }
        return EffortCalloutDecision {
            show: opus_config_enabled,
            mark_v2_dismissed: false,
        };
    }
    if is_max_subscriber || is_team_subscriber {
        return EffortCalloutDecision {
            show: opus_config_enabled,
            mark_v2_dismissed: false,
        };
    }
    EffortCalloutDecision {
        show: false,
        mark_v2_dismissed: true,
    }
}

/// Maps to: CC `EffortCallout` `options`.
pub fn effort_callout_options() -> Vec<SelectOptionData> {
    vec![
        SelectOptionData {
            label: String::new(),
            value: "medium".to_string(),
            ..SelectOptionData::default()
        },
        SelectOptionData {
            label: String::new(),
            value: "high".to_string(),
            ..SelectOptionData::default()
        },
        SelectOptionData {
            label: String::new(),
            value: "low".to_string(),
            ..SelectOptionData::default()
        },
    ]
}

/// Maps to CC EffortCallout.tsx#EffortOptionLabel and EffortIndicatorSymbol.
fn effort_option_label(level: &str, text: &str, suggestion: Color) -> Vec<StyledSegment> {
    let mut symbol = StyledSegment::new(effort_level_to_symbol(level));
    symbol.styles.color = Some(suggestion);
    vec![symbol, StyledSegment::new(format!(" {text}"))]
}

#[derive(Default, Props)]
pub struct EffortCalloutProps {
    pub model: String,
    pub focused_index: usize,
    pub default_effort_level: Option<String>,
    pub config: Option<OpusDefaultEffortConfig>,
}

/// Maps to: CC `components/EffortCallout.tsx#EffortCallout`.
#[component]
pub fn EffortCallout(props: &EffortCalloutProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let _default_level = props.default_effort_level.as_deref().unwrap_or("high");
    let _ = &props.model;
    let config = props
        .config
        .clone()
        .unwrap_or_else(get_opus_default_effort_config);
    let options = effort_callout_options();
    // getTextContent does not execute EffortOptionLabel (it has no children),
    // so option.label stays empty while this owner supplies the rendered flow.
    let option_labels = BTreeMap::from([
        (
            "medium".to_string(),
            effort_option_label("medium", "Medium (recommended)", theme.suggestion),
        ),
        (
            "high".to_string(),
            effort_option_label("high", "High", theme.suggestion),
        ),
        (
            "low".to_string(),
            effort_option_label("low", "Low", theme.suggestion),
        ),
    ]);
    let focused_index = props.focused_index.min(options.len().saturating_sub(1));

    element! {
        PermissionDialog(title: config.dialog_title) {
            View(flex_direction: FlexDirection::Column, padding_left: 2u32, padding_right: 2u32, padding_top: 1u32, padding_bottom: 1u32) {
                View(flex_direction: FlexDirection::Column, margin_bottom: 1u32) {
                    Text(content: config.dialog_description, wrap: TextWrap::Wrap)
                }
                View(flex_direction: FlexDirection::Row, margin_bottom: 1u32) {
                    Text(content: effort_level_to_symbol("low").to_string(), color: theme.suggestion, wrap: TextWrap::NoWrap)
                    Text(content: " low · ".to_string(), dim: true, wrap: TextWrap::NoWrap)
                    Text(content: effort_level_to_symbol("medium").to_string(), color: theme.suggestion, wrap: TextWrap::NoWrap)
                    Text(content: " medium · ".to_string(), dim: true, wrap: TextWrap::NoWrap)
                    Text(content: effort_level_to_symbol("high").to_string(), color: theme.suggestion, wrap: TextWrap::NoWrap)
                    Text(content: " high".to_string(), dim: true, wrap: TextWrap::NoWrap)
                }
                Select(
                    options: options,
                    option_labels,
                    focused_index: focused_index,
                    visible_option_count: 3usize,
                    visible_from_index: 0usize,
                    layout: SelectLayout::Compact,
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
    fn should_show_effort_callout_matches_official_audience_branches() {
        assert_eq!(
            should_show_effort_callout_with_state(
                "claude-sonnet-4-6",
                false,
                9,
                true,
                false,
                false,
                false,
                true
            ),
            EffortCalloutDecision::default()
        );
        assert_eq!(
            should_show_effort_callout_with_state(
                "claude-opus-4-6",
                false,
                1,
                true,
                false,
                false,
                false,
                true
            ),
            EffortCalloutDecision {
                show: false,
                mark_v2_dismissed: true
            }
        );
        assert_eq!(
            should_show_effort_callout_with_state(
                "claude-opus-4-6",
                false,
                9,
                true,
                true,
                false,
                false,
                true
            ),
            EffortCalloutDecision {
                show: false,
                mark_v2_dismissed: true
            }
        );
        assert_eq!(
            should_show_effort_callout_with_state(
                "claude-opus-4-6",
                false,
                9,
                true,
                false,
                false,
                false,
                true
            ),
            EffortCalloutDecision {
                show: true,
                mark_v2_dismissed: false
            }
        );
        assert_eq!(
            should_show_effort_callout_with_state(
                "claude-opus-4-6",
                false,
                9,
                false,
                false,
                true,
                false,
                false
            ),
            EffortCalloutDecision {
                show: false,
                mark_v2_dismissed: false
            }
        );
        assert_eq!(
            should_show_effort_callout_with_state(
                "claude-opus-4-6",
                false,
                9,
                false,
                false,
                false,
                false,
                true
            ),
            EffortCalloutDecision {
                show: false,
                mark_v2_dismissed: true
            }
        );
    }

    #[test]
    fn effort_callout_options_match_official_order_and_copy() {
        let options = effort_callout_options();
        assert_eq!(
            options
                .iter()
                .map(|option| option.value.as_str())
                .collect::<Vec<_>>(),
            ["medium", "high", "low"]
        );
        assert!(
            options.iter().all(|option| option.label.is_empty()),
            "source getTextContent does not execute EffortOptionLabel"
        );
    }

    #[test]
    fn effort_callout_renders_dialog_description_symbols_and_options() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                EffortCallout(model: "claude-opus-4-6".to_string())
            }
        }
        .render(Some(140))
        .to_string();

        assert!(
            text.contains("We recommend medium effort for Opus"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("Effort determines how long Claude thinks"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Medium (recommended)"), "canvas=\n{text}");
        assert!(text.contains("High"), "canvas=\n{text}");
        assert!(text.contains("Low"), "canvas=\n{text}");
    }

    #[test]
    fn effort_select_keeps_unfocused_symbol_color_and_compact_rows() {
        let current_theme = *theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(current_theme)) {
                EffortCallout(model: "claude-opus-4-6".to_string())
            }
        }
        .render(Some(140));
        let text = canvas.to_string();
        let rows = text.lines().collect::<Vec<_>>();
        let row = rows
            .iter()
            .position(|line| line.contains("2. ") && line.contains("High"))
            .expect("compact second option");
        assert!(
            rows[row - 1].contains("1. ") && rows[row - 1].contains("Medium (recommended)"),
            "{text}"
        );
        let symbol = effort_level_to_symbol("high");
        let offset = rows[row].find(symbol).unwrap();
        let column = unicode_width::UnicodeWidthStr::width(&rows[row][..offset]);
        assert_eq!(
            canvas.resolved_text_style(column, row).unwrap().color,
            Some(current_theme.suggestion)
        );
        let offset = rows[row].find("High").unwrap();
        let column = unicode_width::UnicodeWidthStr::width(&rows[row][..offset]);
        assert_eq!(canvas.resolved_text_style(column, row).unwrap().color, None);
    }
}
