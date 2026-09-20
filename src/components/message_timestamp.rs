//! Maps to: CC `components/MessageTimestamp.tsx`.

use crate::types::message::{AssistantContent, Message};
use chrono::{DateTime, Local, Utc};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct MessageTimestampProps {
    pub message: Option<Message>,
    pub is_transcript_mode: bool,
}

/// Maps to: CC `MessageTimestamp` `shouldShowTimestamp` predicate.
pub fn message_timestamp_text(
    message: Option<&Message>,
    is_transcript_mode: bool,
) -> Option<String> {
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
    Some(format_timestamp_for_message(message.timestamp))
}

/// Maps to: CC `new Date(timestamp).toLocaleTimeString('en-US', ...)`.
pub fn format_timestamp_for_message(timestamp: DateTime<Utc>) -> String {
    timestamp
        .with_timezone(&Local)
        .format("%I:%M %p")
        .to_string()
}

/// Maps to: CC `components/MessageTimestamp.tsx#MessageTimestamp`.
#[component]
pub fn MessageTimestamp(
    props: &MessageTimestampProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let Some(text) = message_timestamp_text(props.message.as_ref(), props.is_transcript_mode)
    else {
        return element! { View(width: 0u32, height: 0u32) };
    };
    let min_width = unicode_width::UnicodeWidthStr::width(text.as_str()) as u32;

    element! {
        View(min_width: min_width) {
            Text(content: text, color: theme.inactive, dim: true, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{AssistantMessage, StopReason};
    use crate::utils::theme;

    fn assistant_with_text() -> Message {
        Message::Assistant(AssistantMessage {
            uuid: uuid::Uuid::new_v4().to_string(),
            timestamp: DateTime::parse_from_rfc3339("2026-07-09T12:34:00Z")
                .unwrap()
                .with_timezone(&Utc),
            content: vec![AssistantContent::Text("hello".to_string())],
            model: Some("claude-sonnet-4".to_string()),
            stop_reason: Some(StopReason::EndTurn),
            usage: None,
        })
    }

    #[test]
    fn message_timestamp_predicate_matches_official_transcript_assistant_text_gate() {
        let message = assistant_with_text();
        assert!(message_timestamp_text(Some(&message), true).is_some());
        assert_eq!(message_timestamp_text(Some(&message), false), None);

        let no_text = Message::Assistant(AssistantMessage {
            uuid: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            content: vec![AssistantContent::Thinking {
                text: "thought".to_string(),
                signature: String::new(),
            }],
            model: None,
            stop_reason: None,
            usage: None,
        });
        assert_eq!(message_timestamp_text(Some(&no_text), true), None);
    }

    #[test]
    fn message_timestamp_component_renders_time_in_transcript_mode() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                MessageTimestamp(message: Some(assistant_with_text()), is_transcript_mode: true)
            }
        }
        .render(Some(80))
        .to_string();

        assert!(text.contains(":"), "canvas=\n{text}");
        assert!(
            text.contains("AM") || text.contains("PM"),
            "canvas=\n{text}"
        );
    }
}
