//! Maps to: CC `components/messages/UserPromptMessage.tsx:1-128`.

use super::highlighted_thinking_text::HighlightedThinkingText;
use crate::components::message_actions::MessageActionsSelectedContext;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

const MAX_DISPLAY_CHARS: usize = 10_000;
const TRUNCATE_HEAD_CHARS: usize = 2_500;
const TRUNCATE_TAIL_CHARS: usize = 2_500;

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

fn utf16_slice(text: &str, start: usize, end: usize) -> String {
    let units = text.encode_utf16().collect::<Vec<_>>();
    String::from_utf16_lossy(&units[start.min(units.len())..end.min(units.len())])
}

pub fn truncate_prompt_for_display(text: &str) -> String {
    let length = utf16_len(text);
    if length <= MAX_DISPLAY_CHARS {
        return text.to_string();
    }
    let head = utf16_slice(text, 0, TRUNCATE_HEAD_CHARS);
    let tail_start = length.saturating_sub(TRUNCATE_TAIL_CHARS);
    let tail = utf16_slice(text, tail_start, length);
    let hidden_section = utf16_slice(text, TRUNCATE_HEAD_CHARS, tail_start);
    let hidden_lines = hidden_section.matches('\n').count();
    format!("{head}\n… +{hidden_lines} lines …\n{tail}")
}

#[derive(Default, Props)]
pub struct UserPromptMessageProps {
    pub content: String,
    pub add_margin: bool,
    pub is_transcript_mode: bool,
    /// Explicit projection of the upstream KAIROS/AppState gate.
    pub use_brief_layout: bool,
    pub timestamp: Option<String>,
}

#[component]
pub fn UserPromptMessage(
    props: &UserPromptMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    if props.content.is_empty() {
        return element! { Fragment }.into_any();
    }

    let is_selected = hooks
        .try_use_context::<MessageActionsSelectedContext>()
        .is_some_and(|context| context.0);
    let use_brief_layout = props.use_brief_layout && !props.is_transcript_mode;
    let display_text = truncate_prompt_for_display(&props.content);
    let background = if is_selected {
        Some(theme.message_actions_bg)
    } else if use_brief_layout {
        None
    } else {
        Some(theme.user_message_bg)
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
            padding_right: if use_brief_layout { 0u32 } else { 1u32 },
            background_color: background,
        ) {
            HighlightedThinkingText(
                text: display_text,
                use_brief_layout: use_brief_layout,
                timestamp: if use_brief_layout { props.timestamp.clone() } else { None },
            )
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_truncation_keeps_official_utf16_head_tail_and_line_count() {
        let text = format!("{}\n{}\nquestion", "a".repeat(6_000), "b".repeat(6_000));
        let display = truncate_prompt_for_display(&text);
        assert!(display.starts_with(&"a".repeat(TRUNCATE_HEAD_CHARS)));
        assert!(display.contains("… +1 lines …"));
        assert!(display.ends_with("question"));
        assert!(utf16_len(&display) < MAX_DISPLAY_CHARS);
    }

    #[test]
    fn prompt_truncation_counts_astral_text_like_javascript_length() {
        let text = "😀".repeat(5_001);
        let display = truncate_prompt_for_display(&text);
        assert!(display.contains("… +0 lines …"));
    }

    #[test]
    fn selected_and_brief_background_rules_match_official_parent() {
        let theme = *crate::utils::theme::current();
        let selected = element! {
            ContextProvider(value: Context::owned(theme)) {
                ContextProvider(value: Context::owned(MessageActionsSelectedContext(true))) {
                    UserPromptMessage(content: "hello".to_string(), use_brief_layout: true)
                }
            }
        }
        .render(Some(80));
        assert_eq!(
            selected.cell(0, 0).and_then(|cell| cell.background_color),
            Some(theme.message_actions_bg)
        );

        let brief = element! {
            ContextProvider(value: Context::owned(theme)) {
                UserPromptMessage(content: "hello".to_string(), use_brief_layout: true)
            }
        }
        .render(Some(80));
        assert_eq!(
            brief.cell(0, 0).and_then(|cell| cell.background_color),
            None
        );
        assert!(brief.to_string().contains("You"));
    }

    #[test]
    fn transcript_mode_suppresses_brief_layout() {
        let text = element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                UserPromptMessage(
                    content: "hello".to_string(),
                    use_brief_layout: true,
                    is_transcript_mode: true,
                )
            }
        }
        .render(Some(80))
        .to_string();
        assert!(text.contains("❯ hello"));
        assert!(!text.contains("You"));
    }
}
