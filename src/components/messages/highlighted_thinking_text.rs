//! Maps to: CC `components/messages/HighlightedThinkingText.tsx:1-91`.

use crate::components::message_actions::MessageActionsSelectedContext;
use crate::utils::format_brief_timestamp::format_brief_timestamp;
use crate::utils::theme::Theme;
use crate::utils::thinking::{
    find_thinking_trigger_positions, get_rainbow_color, is_ultrathink_enabled,
};
use chrono::Local;
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueuedMessageContext {
    pub is_queued: bool,
}

/// Deterministic rendering seam for the build-time ULTRATHINK feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UltrathinkHighlightOverride(pub bool);

#[derive(Default, Props)]
pub struct HighlightedThinkingTextProps {
    pub text: String,
    pub use_brief_layout: bool,
    pub timestamp: Option<String>,
}

fn highlighted_contents(
    text: &str,
    theme: &Theme,
    enabled: bool,
    pointer: Color,
) -> Vec<MixedTextContent> {
    let mut contents = vec![MixedTextContent::new("❯ ").color(pointer)];
    let triggers = if enabled {
        find_thinking_trigger_positions(text)
    } else {
        Vec::new()
    };
    if triggers.is_empty() {
        contents.push(MixedTextContent::new(text).color(theme.text));
        return contents;
    }

    let mut cursor = 0usize;
    for trigger in triggers {
        if trigger.start > cursor {
            contents.push(MixedTextContent::new(&text[cursor..trigger.start]).color(theme.text));
        }
        for (index, character) in text[trigger.start..trigger.end].chars().enumerate() {
            contents.push(
                MixedTextContent::new(character).color(get_rainbow_color(theme, index, false)),
            );
        }
        cursor = trigger.end;
    }
    if cursor < text.len() {
        contents.push(MixedTextContent::new(&text[cursor..]).color(theme.text));
    }
    contents
}

/// Brief-layout label or normal pointer/rainbow prompt text.
#[component]
pub fn HighlightedThinkingText(
    props: &HighlightedThinkingTextProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let is_queued = hooks
        .try_use_context::<QueuedMessageContext>()
        .is_some_and(|context| context.is_queued);
    let is_selected = hooks
        .try_use_context::<MessageActionsSelectedContext>()
        .is_some_and(|context| context.0);

    if props.use_brief_layout {
        let timestamp = props
            .timestamp
            .as_deref()
            .map(|value| format_brief_timestamp(value, Local::now()))
            .unwrap_or_default();
        let label_color = if is_queued {
            theme.subtle
        } else {
            theme.brief_label_you
        };
        let text_color = if is_queued { theme.subtle } else { theme.text };
        let mut label = vec![MixedTextContent::new("You").color(label_color)];
        if !timestamp.is_empty() {
            label.push(
                MixedTextContent::new(format!(" {timestamp}"))
                    .color(theme.text)
                    .weight(Weight::Light),
            );
        }
        return element! {
            View(flex_direction: FlexDirection::Column, padding_left: 2u32) {
                MixedText(contents: label, wrap: TextWrap::NoWrap)
                Text(content: props.text.clone(), color: text_color)
            }
        }
        .into_any();
    }

    let highlight_enabled = hooks
        .try_use_context::<UltrathinkHighlightOverride>()
        .map(|override_| override_.0)
        .unwrap_or_else(is_ultrathink_enabled);
    let pointer = if is_selected {
        theme.suggestion
    } else {
        theme.subtle
    };
    let contents = highlighted_contents(&props.text, &theme, highlight_enabled, pointer);
    element! { MixedText(contents: contents, wrap: TextWrap::Wrap) }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_layout_uses_subtle_pointer_and_plain_text_without_trigger() {
        let theme = *crate::utils::theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(theme)) {
                HighlightedThinkingText(text: "hello".to_string())
            }
        }
        .render(Some(40));
        assert_eq!(canvas.to_string().trim_end(), "❯ hello");
        assert_eq!(
            canvas.resolved_text_style(0, 0).unwrap().color,
            Some(theme.subtle)
        );
        assert_eq!(
            canvas.resolved_text_style(2, 0).unwrap().color,
            Some(theme.text)
        );
    }

    #[test]
    fn selected_pointer_and_static_rainbow_match_official_segments() {
        let theme = *crate::utils::theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(theme)) {
                ContextProvider(value: Context::owned(MessageActionsSelectedContext(true))) {
                    ContextProvider(value: Context::owned(UltrathinkHighlightOverride(true))) {
                        HighlightedThinkingText(text: "try ultrathink now".to_string())
                    }
                }
            }
        }
        .render(Some(80));
        assert_eq!(
            canvas.resolved_text_style(0, 0).unwrap().color,
            Some(theme.suggestion)
        );
        let start = "❯ try ".chars().count();
        assert_eq!(
            canvas.resolved_text_style(start, 0).unwrap().color,
            Some(theme.rainbow_red)
        );
        assert_eq!(
            canvas.resolved_text_style(start + 7, 0).unwrap().color,
            Some(theme.rainbow_red)
        );
    }

    #[test]
    fn brief_layout_uses_you_label_queue_color_and_timestamp() {
        let theme = *crate::utils::theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(theme)) {
                ContextProvider(value: Context::owned(QueuedMessageContext { is_queued: true })) {
                    HighlightedThinkingText(
                        text: "hello".to_string(),
                        use_brief_layout: true,
                        timestamp: Some("2026-01-02T13:30:00Z".to_string()),
                    )
                }
            }
        }
        .render(Some(80));
        let text = canvas.to_string();
        assert!(text.contains("You"));
        assert!(text.contains("hello"));
        assert_eq!(
            canvas.resolved_text_style(2, 0).unwrap().color,
            Some(theme.subtle)
        );
    }
}
