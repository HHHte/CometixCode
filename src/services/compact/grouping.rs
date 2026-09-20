//! API-round message grouping for compaction.
//!
//! Maps to CC `services/compact/grouping.ts:1-52`.

use crate::types::message::{AssistantContent, Message};

fn assistant_api_message_id(message: &Message) -> Option<&str> {
    let Message::Assistant(assistant) = message else {
        return None;
    };
    assistant.content.iter().find_map(|content| match content {
        AssistantContent::MessageIdentity(identity) => identity.api_message_id.as_deref(),
        _ => None,
    })
}

/// Maps to CC `services/compact/grouping.ts` `groupMessagesByApiRound(...)`.
pub fn group_messages_by_api_round(messages: &[Message]) -> Vec<Vec<Message>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    let mut last_assistant_id: Option<String> = None;

    for message in messages {
        if matches!(message, Message::Assistant(_)) {
            let current_id = assistant_api_message_id(message).map(str::to_string);
            if current_id.as_deref() != last_assistant_id.as_deref() && !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
            last_assistant_id = current_id;
        }
        current.push(message.clone());
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::{
        AssistantMessage, AssistantMessageIdentity, UserContent, UserMessage,
    };

    fn assistant(api_id: &str, text: &str) -> Message {
        Message::Assistant(AssistantMessage {
            uuid: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            content: vec![
                AssistantContent::Text(text.to_string()),
                AssistantContent::MessageIdentity(AssistantMessageIdentity {
                    request_id: Some("req-shared".to_string()),
                    api_message_id: Some(api_id.to_string()),
                    ..Default::default()
                }),
            ],
            model: None,
            stop_reason: None,
            usage: None,
        })
    }

    fn user(text: &str) -> Message {
        Message::User(UserMessage {
            uuid: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            content: vec![UserContent::Text(text.to_string())],
            is_compact_summary: false,
            plan_content: None,
            image_paste_ids: None,
            is_visible_in_transcript_only: false,
            mcp_meta: None,
            source_tool_assistant_uuid: None,
            permission_mode: None,
            origin: None,
            summarize_metadata: None,
        })
    }

    #[test]
    fn identity_less_legacy_assistants_match_javascript_undefined_grouping() {
        let assistant_without_identity = |text: &str| {
            Message::Assistant(AssistantMessage {
                uuid: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                content: vec![AssistantContent::Text(text.to_string())],
                model: None,
                stop_reason: None,
                usage: None,
            })
        };
        let groups = group_messages_by_api_round(&[
            user("prompt"),
            assistant_without_identity("one"),
            user("result"),
            assistant_without_identity("two"),
        ]);
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn same_api_message_chunks_stay_in_one_round_and_new_id_starts_next() {
        let groups = group_messages_by_api_round(&[
            user("prompt"),
            assistant("msg_1", "tool one"),
            user("tool result"),
            assistant("msg_1", "tool two"),
            user("tool result two"),
            assistant("msg_2", "done"),
        ]);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].len(), 1); // preamble before first assistant
        assert_eq!(groups[1].len(), 4); // both msg_1 chunks + results
        assert_eq!(groups[2].len(), 1);
    }
}
