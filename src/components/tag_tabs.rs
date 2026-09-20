//! Maps to: CC `components/TagTabs.tsx`.
//! Header tabs used by `/resume` LogSelector for tag filtering. This component
//! is deliberately read-only: it only renders tag labels and the selected tab;
//! keyboard state lives in `LogSelector`, matching the official split.

use crate::utils::theme::Theme;
use iocraft::prelude::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const ALL_TAB_LABEL: &str = "All";
const TAB_PADDING: usize = 2;
const HASH_PREFIX_LENGTH: usize = 1;
const LEFT_ARROW_PREFIX: &str = "← ";
const RIGHT_HINT_WITH_COUNT_PREFIX: &str = "→";
const RIGHT_HINT_SUFFIX: &str = " (tab to cycle)";
const RIGHT_HINT_NO_COUNT: &str = "(tab to cycle)";
const MAX_OVERFLOW_DIGITS: usize = 2;

fn get_tab_width(tab: &str, max_width: Option<usize>) -> usize {
    if tab == ALL_TAB_LABEL {
        return ALL_TAB_LABEL.len() + TAB_PADDING;
    }
    let tag_width = UnicodeWidthStr::width(tab);
    let effective_tag_width = max_width
        .map(|width| tag_width.min(width.saturating_sub(TAB_PADDING + HASH_PREFIX_LENGTH)))
        .unwrap_or(tag_width);
    effective_tag_width + TAB_PADDING + HASH_PREFIX_LENGTH
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return text.graphemes(true).next().unwrap_or("…").to_string();
    }

    let mut width = 0usize;
    let mut out = String::new();
    for grapheme in text.graphemes(true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme).max(1);
        if width + grapheme_width > max_width.saturating_sub(1) {
            break;
        }
        out.push_str(grapheme);
        width += grapheme_width;
    }
    out.push('…');
    out
}

fn truncate_tag(tag: &str, max_width: usize) -> String {
    let available_for_tag = max_width.saturating_sub(TAB_PADDING + HASH_PREFIX_LENGTH);
    if UnicodeWidthStr::width(tag) <= available_for_tag {
        return tag.to_string();
    }
    truncate_to_width(tag, available_for_tag.max(1))
}

pub(crate) fn visible_tab_window(
    tab_widths: &[usize],
    selected_index: usize,
    max_tabs_width: usize,
) -> (usize, usize) {
    if tab_widths.is_empty() {
        return (0, 0);
    }

    let selected_index = selected_index.min(tab_widths.len() - 1);
    let total_width = tab_widths
        .iter()
        .enumerate()
        .map(|(idx, width)| width + usize::from(idx + 1 < tab_widths.len()))
        .sum::<usize>();
    if total_width <= max_tabs_width {
        return (0, tab_widths.len());
    }

    let left_arrow_width = LEFT_ARROW_PREFIX.len() + MAX_OVERFLOW_DIGITS + 1;
    let effective_max_width = max_tabs_width.saturating_sub(left_arrow_width).max(1);
    let mut start = selected_index;
    let mut end = selected_index + 1;
    let mut window_width = tab_widths[selected_index];

    while start > 0 || end < tab_widths.len() {
        let can_left = start > 0;
        let can_right = end < tab_widths.len();

        if can_left {
            let left_width = tab_widths[start - 1] + 1;
            if window_width + left_width <= effective_max_width {
                start -= 1;
                window_width += left_width;
                continue;
            }
        }

        if can_right {
            let right_width = tab_widths[end] + 1;
            if window_width + right_width <= effective_max_width {
                end += 1;
                window_width += right_width;
                continue;
            }
        }

        break;
    }

    (start, end)
}

#[derive(Default, Props)]
pub struct TagTabsProps {
    pub tabs: Vec<String>,
    pub selected_index: usize,
    pub available_width: usize,
    pub show_all_projects: bool,
}

#[component]
pub fn TagTabs(props: &TagTabsProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let resume_label = if props.show_all_projects {
        "Resume (All Projects)"
    } else {
        "Resume"
    };
    let resume_label_width = resume_label.len() + 1;
    let right_hint_width =
        (RIGHT_HINT_WITH_COUNT_PREFIX.len() + MAX_OVERFLOW_DIGITS + RIGHT_HINT_SUFFIX.len())
            .max(RIGHT_HINT_NO_COUNT.len());
    let max_tabs_width = props
        .available_width
        .saturating_sub(resume_label_width + right_hint_width + 2);
    let safe_selected_index = props.selected_index.min(props.tabs.len().saturating_sub(1));
    let max_single_tab_width = 20usize.max(max_tabs_width / 2);
    let tab_widths = props
        .tabs
        .iter()
        .map(|tab| get_tab_width(tab, Some(max_single_tab_width)))
        .collect::<Vec<_>>();
    let (start, end) = visible_tab_window(&tab_widths, safe_selected_index, max_tabs_width);
    let hidden_left = start;
    let hidden_right = props.tabs.len().saturating_sub(end);
    let visible_tabs = props.tabs[start..end].to_vec();

    element! {
        View(flex_direction: FlexDirection::Row, flex_shrink: 0.0f32) {
            View(margin_right: 1u32) {
                Text(content: resume_label, color: theme.suggestion, wrap: TextWrap::NoWrap)
            }
            #(if hidden_left > 0 {
                Some(element! {
                    View(margin_right: 1u32) {
                        Text(content: format!("{LEFT_ARROW_PREFIX}{hidden_left}"), color: theme.inactive, wrap: TextWrap::NoWrap)
                    }
                })
            } else { None })
            #(visible_tabs.into_iter().enumerate().map(|(offset, tab)| {
                let actual_index = start + offset;
                let is_selected = actual_index == safe_selected_index;
                let display_text = if tab == ALL_TAB_LABEL {
                    tab.clone()
                } else {
                    format!("#{}", truncate_tag(&tab, max_single_tab_width.saturating_sub(TAB_PADDING)))
                };
                let label = format!(" {display_text} ");
                if is_selected {
                    element! {
                        View(margin_right: 1u32, background_color: theme.suggestion) {
                            Text(content: label, color: theme.inverse_text, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                        }
                    }
                } else {
                    element! {
                        View(margin_right: 1u32) {
                            Text(content: label, wrap: TextWrap::NoWrap)
                        }
                    }
                }
            }))
            #(if hidden_right > 0 {
                Some(element! {
                    Text(content: format!("{RIGHT_HINT_WITH_COUNT_PREFIX}{hidden_right}{RIGHT_HINT_SUFFIX}"), color: theme.inactive, wrap: TextWrap::NoWrap)
                })
            } else {
                Some(element! {
                    Text(content: RIGHT_HINT_NO_COUNT.to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_window_keeps_selected_tab_visible_when_overflowing() {
        let widths = vec![6, 12, 12, 12, 12, 6];
        let (start, end) = visible_tab_window(&widths, 4, 25);
        assert!(start <= 4 && 4 < end);
        assert!(end < widths.len());
    }

    #[test]
    fn tag_truncation_preserves_unicode_width_budget() {
        let tag = truncate_tag("非常非常long-tag", 8);
        assert!(UnicodeWidthStr::width(tag.as_str()) <= 8);
    }
}
