//! Maps to: CC `components/MessageModel.tsx`.

use crate::types::message::{AssistantContent, Message};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct MessageModelProps {
    pub message: Option<Message>,
    pub is_transcript_mode: bool,
}

/// Maps to: CC `MessageModel` `shouldShowModel` predicate.
pub fn message_model_text(message: Option<&Message>, is_transcript_mode: bool) -> Option<String> {
    if !is_transcript_mode {
        return None;
    }
    let Message::Assistant(message) = message? else {
        return None;
    };
    if !message
        .content
        .iter()
        .any(|content| matches!(content, AssistantContent::Text(_)))
    {
        return None;
    }
    message
        .model
        .as_deref()
        .filter(|model| !model.is_empty())
        .map(ToString::to_string)
}

/// Maps to: CC `components/MessageModel.tsx#MessageModel`.
#[component]
pub fn MessageModel(props: &MessageModelProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let Some(model) = message_model_text(props.message.as_ref(), props.is_transcript_mode) else {
        return element! { View(width: 0u32, height: 0u32) };
    };
    let min_width = unicode_width::UnicodeWidthStr::width(model.as_str()) as u32 + 8;

    element! {
        View(min_width: min_width) {
            Text(content: model, color: theme.inactive, dim: true, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{AssistantMessage, StopReason};
    use crate::utils::theme;
    use chrono::Utc;

    fn assistant(model: Option<&str>, has_text: bool) -> Message {
        Message::Assistant(AssistantMessage {
            uuid: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            content: if has_text {
                vec![AssistantContent::Text("hello".to_string())]
            } else {
                vec![AssistantContent::Thinking {
                    text: "thought".to_string(),
                    signature: String::new(),
                }]
            },
            model: model.map(ToString::to_string),
            stop_reason: Some(StopReason::EndTurn),
            usage: None,
        })
    }

    #[test]
    fn message_model_predicate_matches_official_transcript_model_text_gate() {
        let message = assistant(Some("claude-sonnet-4"), true);
        assert_eq!(
            message_model_text(Some(&message), true).as_deref(),
            Some("claude-sonnet-4")
        );
        assert_eq!(message_model_text(Some(&message), false), None);
        assert_eq!(message_model_text(Some(&assistant(None, true)), true), None);
        assert_eq!(
            message_model_text(Some(&assistant(Some("claude"), false)), true),
            None
        );
    }

    #[test]
    fn message_model_component_renders_model_name() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                MessageModel(message: Some(assistant(Some("claude-sonnet-4"), true)), is_transcript_mode: true)
            }
        }
        .render(Some(80))
        .to_string();

        assert!(text.contains("claude-sonnet-4"), "canvas=\n{text}");
    }
}
