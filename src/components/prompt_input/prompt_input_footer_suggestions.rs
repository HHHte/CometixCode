//! Maps to: CC `components/PromptInput/PromptInputFooterSuggestions.tsx`.
//!
//! Command suggestion filtering lives in
//! `utils/suggestions/command_suggestions.rs`, matching official
//! `utils/suggestions/commandSuggestions.ts`.
//!
//! Row layout, from `SuggestionItemRow` (:127-174) — the non-unified branch:
//!   - name column: `min(maxColumnWidth ?? width(displayText) + 5,
//!     floor(columns * 0.4))`, the name truncated to `width - 2` then padded
//!     out to the full column
//!   - description column: `columns - nameColumn - tagWidth - 4`, whitespace
//!     collapsed to single spaces, truncated with `…`
//!   - the row is `<Text wrap="truncate">`: one line, never wrapped
//!
//! All widths are terminal display widths, not byte or `char` counts. CC uses
//! `ink/stringWidth.ts`; the Rust equivalent is `unicode_width`, which agrees
//! with it on everything except ANSI-bearing strings (CC strips ANSI first).
//! Suggestion text carries no ANSI, so `unicode_width` is exact here.
//!
//! Unified aggregation is owned by `hooks/unified_suggestions.rs`; this file
//! only renders the source-shaped row. Optional `tag` and `color` are carried
//! on `SuggestionItem` so command producers do not need a second UI protocol.

use crate::commands::Command;
use crate::utils::suggestions::command_suggestions::generate_command_suggestions;
use crate::utils::truncate::{truncate_path_middle, truncate_to_width};
use iocraft::prelude::*;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

/// Maps to: CC `PromptInputFooterSuggestions.tsx:9-16` `SuggestionItem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionItem {
    pub id: String,
    pub display_text: String,
    /// Maps to the source optional `tag` label (for example `workflow`).
    pub tag: Option<String>,
    pub command_text: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    /// Maps to the source optional `color` theme key.  Keeping the typed key
    /// here lets the footer resolve it through the shared Theme owner rather
    /// than serializing a second color protocol into metadata.
    pub color: Option<crate::utils::theme::ThemeColorKey>,
}

/// Maps to: CC `PromptInputFooterSuggestions.tsx:28` `OVERLAY_MAX_ITEMS`.
pub const OVERLAY_MAX_ITEMS: usize = 5;

/// Maps to: CC `PromptInputFooterSuggestions.tsx:199-201` `maxVisibleItems`.
///
/// Overlay mode is a fixed 5 — the floating box sits over the ScrollBox, so
/// terminal height is not the constraint. Inline mode leaves room for the
/// prompt itself, and always shows at least one row.
///
/// The caller sizing the footer must use this same number: a mismatch between
/// the reserved height and the rows actually rendered leaves ghost rows behind.
pub fn max_visible_items(rows: u16, overlay: bool) -> usize {
    if overlay {
        return OVERLAY_MAX_ITEMS;
    }
    6.min((rows as usize).saturating_sub(3).max(1))
}

/// Generate command suggestions from input.
/// Maps to: CC `utils/suggestions/commandSuggestions.ts`
/// `generateCommandSuggestions()`.
pub fn generate_suggestions(input: &str, commands: &[Command]) -> Vec<SuggestionItem> {
    generate_command_suggestions(input, commands)
}

/// Maps to: CC `SuggestionItemRow` (:127-174), non-unified branch — the row
/// text and the widths behind it.
///
/// Returned as one string because all three of CC's `<Text>` segments render
/// identically while `tag` is unreachable: name and description both take
/// `suggestion` when selected and `dimColor` otherwise, and the tag segment
/// (the only permanently-dim one) is never emitted. Wiring `tag` up means
/// splitting this into a row of three `Text`s.
fn suggestion_row_text(item: &SuggestionItem, columns: usize, max_column_width: usize) -> String {
    // CC :129-133 — cap the name column at 40% of the terminal.
    let max_name_width = (columns * 40) / 100;
    let display_text_width = max_column_width.min(max_name_width);

    // CC :139-145 — truncate the name to the column, then pad it out to the
    // full width so descriptions line up.
    let mut display_text = item.display_text.clone();
    let reserved = display_text_width.saturating_sub(2);
    if UnicodeWidthStr::width(display_text.as_str()) > reserved {
        display_text = truncate_to_width(&display_text, reserved);
    }
    let padding = display_text_width.saturating_sub(UnicodeWidthStr::width(display_text.as_str()));
    let padded_display_text = format!("{display_text}{}", " ".repeat(padding));

    // CC :147-152 — tags are permanently dim and consume their own width.
    let tag_text = item
        .tag
        .as_deref()
        .map(|tag| format!("[{tag}] "))
        .unwrap_or_default();
    let tag_width = UnicodeWidthStr::width(tag_text.as_str());
    let description_width = columns
        .saturating_sub(display_text_width)
        .saturating_sub(tag_width)
        .saturating_sub(4);

    // CC :153-159 — skill descriptions can contain newlines, and a multi-line
    // row grows the overlay past its minHeight, leaving ghost rows when the
    // filter narrows. Collapse runs of whitespace to one space before
    // truncating. CC's `/\s+/g` collapses RUNS, not individual characters.
    let description = collapse_whitespace(&item.description);
    let truncated_description = truncate_to_width(&description, description_width);

    format!("{padded_display_text}{tag_text}{truncated_description}")
}

/// Maps to: CC `getIcon` (:34-39).
fn get_icon(item_id: &str) -> &'static str {
    if item_id.starts_with("file-") {
        "+"
    } else if item_id.starts_with("mcp-resource-") {
        "◇"
    } else if item_id.starts_with("agent-") {
        "*"
    } else {
        "+"
    }
}

/// Maps to: CC `isUnifiedSuggestion` (:44-49).
fn is_unified_suggestion(item_id: &str) -> bool {
    item_id.starts_with("file-")
        || item_id.starts_with("mcp-resource-")
        || item_id.starts_with("agent-")
}

/// Maps to: CC `SuggestionItemRow` (:64-124), unified branch (`file-` /
/// `mcp-resource-` / `agent-`).
fn unified_suggestion_row_text(item: &SuggestionItem, columns: usize) -> String {
    let icon = get_icon(&item.id);
    let is_file = item.id.starts_with("file-");
    let is_mcp_resource = item.id.starts_with("mcp-resource-");
    let separator_width = if item.description.is_empty() { 0 } else { 3 };
    let desc_reserve = if item.description.is_empty() {
        0
    } else {
        20.min(UnicodeWidthStr::width(item.description.as_str()))
    };
    let display_width = columns
        .saturating_sub(2)
        .saturating_sub(4)
        .saturating_sub(separator_width)
        .saturating_sub(desc_reserve);
    let display_text = if is_file {
        truncate_path_middle(&item.display_text, display_width)
    } else if is_mcp_resource {
        truncate_to_width(&item.display_text, 30)
    } else {
        item.display_text.clone()
    };
    let available = columns
        .saturating_sub(2)
        .saturating_sub(UnicodeWidthStr::width(display_text.as_str()))
        .saturating_sub(separator_width)
        .saturating_sub(4);
    if item.description.is_empty() {
        format!("{icon} {display_text}")
    } else {
        format!(
            "{icon} {display_text} – {}",
            truncate_to_width(&collapse_whitespace(&item.description), available)
        )
    }
}

/// Maps to: CC `.replace(/\s+/g, ' ')` — every run of whitespace becomes a
/// single space.
fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_whitespace = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                out.push(' ');
                in_whitespace = true;
            }
        } else {
            out.push(ch);
            in_whitespace = false;
        }
    }
    out
}

/// Maps to: CC `PromptInputFooterSuggestions.tsx:214-222` — the visible window.
///
/// `selected` is centred where it can be; the window is clamped so it never
/// runs past either end.
fn visible_range(selected: usize, total: usize, max_visible: usize) -> (usize, usize) {
    let start = selected
        .saturating_sub(max_visible / 2)
        .min(total.saturating_sub(max_visible));
    let end = (start + max_visible).min(total);
    (start, end)
}

#[derive(Default, Props)]
pub struct SuggestionListProps {
    pub items: Arc<Vec<SuggestionItem>>,
    pub selected: i32,
    /// Stable width computed from all visible commands by `use_typeahead`,
    /// matching official `allCommandsMaxWidth`.
    pub max_column_width: Option<usize>,
    /// Maps to: CC `Props.overlay` (:181-186). When set, the list is inside a
    /// `position=absolute` overlay: drop `justifyContent` so the renderer's
    /// y-clamp does not push rows down into the prompt area.
    pub overlay: bool,
}

/// Maps to: CC `PromptInputFooterSuggestions` (:189-246).
#[component]
pub fn SuggestionList(
    props: &SuggestionListProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    // Retained renders can transition between zero and many suggestions. Keep
    // hook topology fixed across both the empty branch and viewport changes.
    let (width, rows) = hooks.use_terminal_size();
    let theme = hooks.use_context::<crate::utils::theme::Theme>();

    // CC :204-206 — nothing to display.
    if props.items.is_empty() {
        return element! { View };
    }

    let total = props.items.len();
    let max_visible = max_visible_items(rows, props.overlay).min(total);

    // CC :209-211 — fall back to the widest visible name when no stable width
    // was supplied. The `+ 5` is CC's gutter between the columns.
    let max_column_width = props.max_column_width.unwrap_or_else(|| {
        props
            .items
            .iter()
            .map(|item| UnicodeWidthStr::width(item.display_text.as_str()))
            .max()
            .unwrap_or(0)
            + 5
    });

    // CC :241 compares ids, so an out-of-range index selects nothing rather
    // than silently highlighting the first row.
    let selected_id = usize::try_from(props.selected)
        .ok()
        .and_then(|index| props.items.get(index))
        .map(|item| item.id.clone());
    let (start, end) = visible_range(
        usize::try_from(props.selected).unwrap_or(0),
        total,
        max_visible,
    );

    element! {
        View(
            flex_direction: FlexDirection::Column,
            // CC :232-235 — inline mode anchors the list to the bottom, next
            // to the prompt; overlay mode must not.
            justify_content: if props.overlay { JustifyContent::FLEX_START } else { JustifyContent::FLEX_END },
        ) {
            #((start..end).map(|idx| {
                let item = &props.items[idx];
                let is_selected = selected_id.as_deref() == Some(item.id.as_str());
                let content = if is_unified_suggestion(&item.id) {
                    unified_suggestion_row_text(item, width as usize)
                } else {
                    suggestion_row_text(item, width as usize, max_column_width)
                };
                element! {
                    // CC :162 — `<Text wrap="truncate">`, and :135-136: the
                    // selected row takes `suggestion`, the rest are dimmed.
                    // CC applies no bold here.
                    Text(
                        content,
                        color: item
                            .color
                            .map(|key| theme.color(key))
                            .or_else(|| is_selected.then_some(theme.suggestion)),
                        dim: !is_selected,
                        wrap: TextWrap::Truncate,
                    )
                }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, display: &str, description: &str) -> SuggestionItem {
        SuggestionItem {
            id: id.to_string(),
            display_text: display.to_string(),
            tag: None,
            command_text: display.to_string(),
            description: description.to_string(),
            metadata: None,
            color: None,
        }
    }

    /// The row is built from display widths, so a multi-byte description is
    /// cut on a character boundary. Slicing by byte index panics here — the
    /// description is 3-byte-per-char CJK and the cut lands mid-character.
    #[test]
    fn multibyte_description_truncates_without_panicking() {
        let it = item(
            "cmd-1",
            "/compact",
            &"压缩当前会话历史记录以节省上下文".repeat(4),
        );
        let row = suggestion_row_text(&it, 60, 20);
        assert!(UnicodeWidthStr::width(row.as_str()) <= 60);
        assert!(row.ends_with('…'), "long descriptions are elided: {row}");
    }

    /// An emoji description exercises the same boundary from the other side:
    /// each emoji is one `char` but two columns wide.
    #[test]
    fn emoji_description_truncates_on_display_width() {
        let it = item("cmd-2", "/x", &"🎉".repeat(40));
        let row = suggestion_row_text(&it, 40, 10);
        assert!(UnicodeWidthStr::width(row.as_str()) <= 40);
    }

    /// CC pads the name column to a fixed width so descriptions align, and
    /// truncates the name itself at `width - 2` (CC :140-142).
    #[test]
    fn name_column_is_padded_and_truncated_like_official() {
        let short = item("a", "/ab", "desc");
        let row = suggestion_row_text(&short, 80, 12);
        assert!(row.starts_with("/ab         "), "padded to 12: {row:?}");

        let long = item("b", "/a-very-long-command-name", "desc");
        let row = suggestion_row_text(&long, 80, 12);
        // Truncated to 12 - 2 = 10 columns, then padded back out to 12.
        assert!(row.starts_with("/a-very-l…  "), "{row:?}");
    }

    /// CC collapses whitespace RUNS to a single space, not each whitespace
    /// character (`/\s+/g`). A skill description with a newline plus indent
    /// must not become a run of spaces.
    #[test]
    fn whitespace_runs_collapse_like_official() {
        assert_eq!(collapse_whitespace("a\n    b"), "a b");
        assert_eq!(collapse_whitespace("a \t\n b"), "a b");
        assert_eq!(collapse_whitespace("plain text"), "plain text");
    }

    /// CC :129 caps the name column at 40% of the terminal even when the
    /// stable width is larger.
    #[test]
    fn name_column_caps_at_forty_percent_like_official() {
        let it = item("c", "/some-command", "the description");
        // 40% of 40 columns = 16, so a requested 30 is clamped to 16.
        let row = suggestion_row_text(&it, 40, 30);
        assert!(row.starts_with("/some-command   "), "{row:?}");
    }

    /// The reported `@../` regression, as data.
    ///
    /// `getPathCompletions` rows carry a bare path as their id, so they render
    /// through this non-unified branch, and CC publishes `maxColumnWidth =
    /// undefined` for them — the footer then sizes the column from the rows it
    /// actually has (:209-211). Handing it the command column instead cuts the
    /// path mid-name, which is what produced `../ansi-behavior-followup…`.
    #[test]
    fn directory_rows_size_from_their_own_width_not_the_command_column() {
        let row = item(
            "../ansi-behavior-followup-0912",
            "../ansi-behavior-followup-0912/",
            "",
        );
        let own_width = UnicodeWidthStr::width("../ansi-behavior-followup-0912/") + 5;
        let sized = suggestion_row_text(&row, 100, own_width);
        assert!(
            sized.starts_with("../ansi-behavior-followup-0912/"),
            "footer fallback keeps the whole path: {sized:?}"
        );

        // The command column is `longest command name + 6`; nothing about it
        // tracks path width.
        let truncated = suggestion_row_text(&row, 100, 28);
        assert!(
            truncated.starts_with("../ansi-behavior-followup…"),
            "command column truncates the path: {truncated:?}"
        );
    }

    /// CC :199-201 — overlay is a fixed 5, inline is `min(6, max(1, rows - 3))`.
    #[test]
    fn visible_item_count_matches_official() {
        assert_eq!(max_visible_items(50, true), 5);
        assert_eq!(max_visible_items(50, false), 6);
        assert_eq!(max_visible_items(9, false), 6);
        assert_eq!(max_visible_items(8, false), 5);
        assert_eq!(max_visible_items(4, false), 1);
        // Degenerate terminals still get one row rather than zero.
        assert_eq!(max_visible_items(1, false), 1);
        assert_eq!(max_visible_items(0, false), 1);
    }

    /// CC :214-222 — the window centres the selection and clamps at both ends.
    #[test]
    fn visible_range_matches_official() {
        assert_eq!(visible_range(0, 10, 5), (0, 5));
        assert_eq!(visible_range(4, 10, 5), (2, 7));
        assert_eq!(visible_range(9, 10, 5), (5, 10));
        // Fewer items than the window: one full range, no underflow.
        assert_eq!(visible_range(1, 3, 3), (0, 3));
    }
}
