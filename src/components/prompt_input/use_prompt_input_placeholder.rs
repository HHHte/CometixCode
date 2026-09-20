//! Maps to: CC `components/PromptInput/usePromptInputPlaceholder.ts:1-77`.

use crate::utils::message_queue_manager::{QueuedCommand, is_queued_command_editable};

pub const NUM_TIMES_QUEUE_HINT_SHOWN: u32 = 3;
pub const MAX_TEAMMATE_NAME_LENGTH: usize = 20;

#[derive(Clone, Debug)]
pub struct PromptPlaceholderInput<'a> {
    pub input: &'a str,
    pub submit_count: usize,
    pub viewing_agent_name: Option<&'a str>,
    pub queued_commands: &'a [QueuedCommand],
    pub queued_command_up_hint_count: u32,
    pub prompt_suggestion_enabled: bool,
    pub proactive_active: bool,
    pub example_command: Option<&'a str>,
}

pub fn prompt_input_placeholder(input: PromptPlaceholderInput<'_>) -> Option<String> {
    if !input.input.is_empty() {
        return None;
    }
    if let Some(name) = input.viewing_agent_name {
        let units = name.encode_utf16().collect::<Vec<_>>();
        let display = if units.len() > MAX_TEAMMATE_NAME_LENGTH {
            format!(
                "{}...",
                String::from_utf16_lossy(&units[..MAX_TEAMMATE_NAME_LENGTH - 3])
            )
        } else {
            name.to_string()
        };
        return Some(format!("Message @{display}…"));
    }
    if input.queued_commands.iter().any(is_queued_command_editable)
        && input.queued_command_up_hint_count < NUM_TIMES_QUEUE_HINT_SHOWN
    {
        return Some("Press up to edit queued messages".to_string());
    }
    if input.submit_count < 1 && input.prompt_suggestion_enabled && !input.proactive_active {
        return input.example_command.map(str::to_string);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base<'a>(queue: &'a [QueuedCommand]) -> PromptPlaceholderInput<'a> {
        PromptPlaceholderInput {
            input: "",
            submit_count: 0,
            viewing_agent_name: None,
            queued_commands: queue,
            queued_command_up_hint_count: 0,
            prompt_suggestion_enabled: true,
            proactive_active: false,
            example_command: Some("Try /help"),
        }
    }
    #[test]
    fn precedence_matches_input_teammate_queue_and_example_order() {
        let queue = vec![QueuedCommand::new("queued", "prompt")];
        let mut data = base(&queue);
        data.input = "x";
        assert_eq!(prompt_input_placeholder(data.clone()), None);
        data.input = "";
        data.viewing_agent_name = Some("a-very-long-background-agent-name");
        assert_eq!(
            prompt_input_placeholder(data.clone()).unwrap(),
            "Message @a-very-long-backg...…"
        );
        data.viewing_agent_name = None;
        assert_eq!(
            prompt_input_placeholder(data.clone()).unwrap(),
            "Press up to edit queued messages"
        );
        data.queued_command_up_hint_count = 3;
        assert_eq!(prompt_input_placeholder(data).as_deref(), Some("Try /help"));
    }
    #[test]
    fn task_notifications_and_meta_commands_are_not_editable() {
        let task = QueuedCommand::new("done", "task-notification");
        let mut meta = QueuedCommand::new("tick", "prompt");
        meta.is_meta = true;
        assert!(!is_queued_command_editable(&task));
        assert!(!is_queued_command_editable(&meta));
    }
}
