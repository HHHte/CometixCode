//! Maps to: CC `components/design-system/FuzzyPicker.tsx`.
//! Rust screens keep filtering and event ownership outside this primitive. This
//! module renders the official main-screen picker chrome: Pane, title, SearchBox,
//! bounded list window, optional preview, match label, and byline hints.

use super::byline::Byline;
use super::keyboard_shortcut_hint::KeyboardShortcutHint;
use super::list_item::ListItem;
use super::pane::Pane;
use crate::components::search_box::SearchBox;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

const DEFAULT_VISIBLE: usize = 8;
const CHROME_ROWS: usize = 10;
const MIN_VISIBLE: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FuzzyPickerItem {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    pub preview: Option<String>,
}

impl FuzzyPickerItem {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            description: None,
            preview: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FuzzyPickerDirection {
    #[default]
    Down,
    Up,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FuzzyPickerPreviewPosition {
    #[default]
    Bottom,
    Right,
}

pub fn first_word(s: &str) -> &str {
    s.split_once(' ').map(|(first, _)| first).unwrap_or(s)
}

pub fn fuzzy_picker_visible_count(requested: usize, rows: usize, has_match_label: bool) -> usize {
    let chrome = CHROME_ROWS + usize::from(has_match_label);
    let available = rows.saturating_sub(chrome);
    requested.max(MIN_VISIBLE).min(available.max(MIN_VISIBLE))
}

pub fn fuzzy_picker_window_start(
    focused_index: usize,
    visible_count: usize,
    total: usize,
) -> usize {
    if total <= visible_count {
        return 0;
    }
    focused_index
        .saturating_sub(visible_count.saturating_sub(1))
        .min(total - visible_count)
}

#[derive(Default, Props)]
pub struct FuzzyPickerProps {
    pub title: String,
    pub placeholder: Option<String>,
    pub query: String,
    pub cursor_offset: Option<usize>,
    pub items: Vec<FuzzyPickerItem>,
    pub focused_index: usize,
    pub visible_count: Option<u32>,
    pub direction: FuzzyPickerDirection,
    pub preview_position: FuzzyPickerPreviewPosition,
    pub empty_message: Option<String>,
    pub match_label: Option<String>,
    pub select_action: Option<String>,
    pub tab_action: Option<String>,
    pub shift_tab_action: Option<String>,
}

#[component]
pub fn FuzzyPicker(props: &FuzzyPickerProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let is_terminal_focused = hooks.use_terminal_focus();
    let (columns, rows) = hooks.use_terminal_size();
    let requested_visible = props
        .visible_count
        .map(|count| count as usize)
        .unwrap_or(DEFAULT_VISIBLE);
    let visible_count = fuzzy_picker_visible_count(
        requested_visible,
        usize::from(rows),
        props.match_label.is_some(),
    );
    let total = props.items.len();
    let focused_index = props.focused_index.min(total.saturating_sub(1));
    let window_start = fuzzy_picker_window_start(focused_index, visible_count, total);
    let visible_items = props
        .items
        .iter()
        .skip(window_start)
        .take(visible_count)
        .cloned()
        .collect::<Vec<_>>();
    let focused = props.items.get(focused_index).cloned();
    let compact = columns < 120;
    let select_action = props
        .select_action
        .clone()
        .unwrap_or_else(|| "select".to_string());
    let enter_action = if compact {
        first_word(&select_action).to_string()
    } else {
        select_action
    };
    let placeholder = props
        .placeholder
        .clone()
        .unwrap_or_else(|| "Type to search…".to_string());
    let empty_text = props
        .empty_message
        .clone()
        .unwrap_or_else(|| "No results".to_string());
    let input_above = props.direction != FuzzyPickerDirection::Up;
    let list_height = visible_count as u32;

    let search_box_above = if input_above {
        Some(element! {
            SearchBox(
                query: props.query.clone(),
                cursor_offset: props.cursor_offset.or_else(|| Some(props.query.chars().count())),
                placeholder: Some(placeholder.clone()),
                is_focused: true,
                is_terminal_focused: is_terminal_focused,
            )
        }.into_any())
    } else {
        None
    };
    let search_box_below = if !input_above {
        Some(element! {
            SearchBox(
                query: props.query.clone(),
                cursor_offset: props.cursor_offset.or_else(|| Some(props.query.chars().count())),
                placeholder: Some(placeholder.clone()),
                is_focused: true,
                is_terminal_focused: is_terminal_focused,
            )
        }.into_any())
    } else {
        None
    };

    let list_block = if visible_items.is_empty() {
        element! {
            View(height: list_height, flex_shrink: 0.0f32) {
                Text(content: empty_text, color: theme.inactive, wrap: TextWrap::NoWrap)
            }
        }
        .into_any()
    } else {
        element! {
            View(
                height: list_height,
                flex_direction: if props.direction == FuzzyPickerDirection::Up { FlexDirection::ColumnReverse } else { FlexDirection::Column },
                flex_shrink: 0.0f32,
            ) {
                #(visible_items.into_iter().enumerate().map(|(index, item)| {
                    let actual_index = window_start + index;
                    let is_focused = actual_index == focused_index;
                    let at_low_edge = index == 0 && window_start > 0;
                    let at_high_edge = index + 1 == visible_count && window_start + visible_count < total;
                    let show_scroll_up = if props.direction == FuzzyPickerDirection::Up { at_high_edge } else { at_low_edge };
                    let show_scroll_down = if props.direction == FuzzyPickerDirection::Up { at_low_edge } else { at_high_edge };
                    element! {
                        ListItem(
                            is_focused: is_focused,
                            show_scroll_up: show_scroll_up,
                            show_scroll_down: show_scroll_down,
                            label: Some(item.label),
                            description: item.description,
                        )
                    }
                }))
            }
        }
        .into_any()
    };

    let preview = focused.and_then(|item| item.preview).map(|preview| {
        element! {
            View(flex_direction: FlexDirection::Column, flex_grow: 1.0f32) {
                Text(content: preview, dim: true, wrap: TextWrap::Wrap)
            }
        }
        .into_any()
    });

    let list_group = if props.preview_position == FuzzyPickerPreviewPosition::Right {
        element! {
            View(flex_direction: FlexDirection::Row, height: list_height, column_gap: 2u32) {
                View(flex_direction: FlexDirection::Column, flex_shrink: 0.0f32) {
                    #(vec![list_block])
                    #(props.match_label.as_ref().map(|label| element! {
                        Text(content: label.clone(), dim: true, wrap: TextWrap::NoWrap)
                    }))
                }
                #(preview)
            }
        }
        .into_any()
    } else {
        element! {
            View(flex_direction: FlexDirection::Column) {
                #(vec![list_block])
                #(props.match_label.as_ref().map(|label| element! {
                    Text(content: label.clone(), dim: true, wrap: TextWrap::NoWrap)
                }))
                #(preview)
            }
        }
        .into_any()
    };

    element! {
        Pane(color: Some(theme.permission)) {
            View(flex_direction: FlexDirection::Column, row_gap: 1u32) {
                Text(content: props.title.clone(), color: theme.permission, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                #(search_box_above)
                #(vec![list_group])
                #(search_box_below)
                Byline {
                    KeyboardShortcutHint(shortcut: "↑/↓".to_string(), action: if compact { "nav".to_string() } else { "navigate".to_string() })
                    KeyboardShortcutHint(shortcut: "Enter".to_string(), action: enter_action)
                    #(props.tab_action.as_ref().map(|action| element! {
                        KeyboardShortcutHint(shortcut: "Tab".to_string(), action: action.clone())
                    }))
                    #(if compact { None } else { props.shift_tab_action.as_ref().map(|action| element! {
                        KeyboardShortcutHint(shortcut: "shift+tab".to_string(), action: action.clone())
                    })})
                    KeyboardShortcutHint(shortcut: "Esc".to_string(), action: "cancel".to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn fuzzy_picker_helpers_match_official_windowing_shape() {
        assert_eq!(first_word("select item"), "select");
        assert_eq!(fuzzy_picker_visible_count(8, 12, false), 2);
        assert_eq!(fuzzy_picker_window_start(7, 3, 10), 5);
        assert_eq!(fuzzy_picker_window_start(1, 3, 2), 0);
    }

    #[test]
    fn fuzzy_picker_renders_pane_search_list_preview_and_hints() {
        let current_theme = *theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(current_theme)) {
                FuzzyPicker(
                    title: "Pick file".to_string(),
                    query: "src".to_string(),
                    focused_index: 2usize,
                    visible_count: Some(2u32),
                    match_label: Some("3 matches".to_string()),
                    tab_action: Some("mention".to_string()),
                    items: vec![
                        FuzzyPickerItem::new("a", "alpha"),
                        FuzzyPickerItem { key: "b".to_string(), label: "beta".to_string(), description: None, preview: None },
                        FuzzyPickerItem { key: "c".to_string(), label: "gamma".to_string(), description: None, preview: Some("preview".to_string()) },
                    ],
                )
            }
        }
        .render(Some(120));
        let text = canvas.to_string();

        assert!(text.contains("Pick file"), "canvas=\n{text}");
        assert!(text.contains("src"), "canvas=\n{text}");
        assert!(text.contains("gamma"), "canvas=\n{text}");
        assert!(text.contains("preview"), "canvas=\n{text}");
        assert!(text.contains("3 matches"), "canvas=\n{text}");
        assert!(text.contains("Tab to mention"), "canvas=\n{text}");
    }
}
