//! Queue dispatch policy.
//!
//! Maps to CC `utils/queueProcessor.ts`.
//! The module owns only dequeue policy. The REPL owns execution and rendering
//! of the returned commands, just as CC's `executeInput` callback owns the
//! actual prompt/bash processing.

use super::message_queue_manager::{
    QueuedCommand, dequeue, dequeue_all_matching, has_commands_in_queue, peek,
};

/// Checks whether a queued command is slash-shaped.
///
/// Maps to CC `utils/queueProcessor.ts:20-29` `isSlashCommand`, which is a private
/// queue-processing predicate rather than a message-queue-manager API. Rust's
/// queue carrier currently stores the normalized text form, so the
/// ContentBlockParam branch in CC is represented by the single string field.
fn is_slash_command(command: &QueuedCommand) -> bool {
    command.value.trim_start().starts_with('/')
}

/// Dequeues one executable group from the main-thread queue.
///
/// Slash and bash commands are isolated. Other commands with the same mode as
/// the highest-priority main-thread command are drained as one batch. Commands
/// addressed to a subagent remain queued for that agent's consumer.
///
/// Maps to CC `utils/queueProcessor.ts:52-89` `processQueueIfReady({ executeInput })`. Rust returns the group
/// because the iocraft REPL cannot pass an async closure into this utility;
/// execution remains in the REPL owner.
pub fn process_queue_if_ready() -> Option<Vec<QueuedCommand>> {
    let is_main_thread = |command: &QueuedCommand| command.agent_id.is_none();
    let next = peek(is_main_thread)?;

    if is_slash_command(&next) || next.mode == "bash" {
        return dequeue(is_main_thread).map(|command| vec![command]);
    }

    let target_mode = next.mode.clone();
    let commands = dequeue_all_matching(|command| {
        is_main_thread(command) && !is_slash_command(command) && command.mode == target_mode
    });
    (!commands.is_empty()).then_some(commands)
}

/// Maps to CC `utils/queueProcessor.ts:93-96` `hasQueuedCommands`.
pub fn has_queued_commands() -> bool {
    has_commands_in_queue()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::message_queue_manager::{
        QueuePriority, TEST_QUEUE_LOCK, clear_command_queue, enqueue,
    };

    #[test]
    fn isolates_slash_and_bash_but_batches_same_mode() {
        let _lock = TEST_QUEUE_LOCK.lock().unwrap();
        clear_command_queue();

        let mut first = QueuedCommand::new("first", "prompt");
        first.priority = QueuePriority::Next;
        enqueue(first);
        enqueue(QueuedCommand::new("second", "prompt"));

        let batch = process_queue_if_ready().expect("prompt batch");
        assert_eq!(batch.len(), 2);

        enqueue(QueuedCommand::new("/help", "prompt"));
        let slash = process_queue_if_ready().expect("slash command");
        assert_eq!(slash.len(), 1);
        assert_eq!(slash[0].value, "/help");

        enqueue(QueuedCommand::new("echo hi", "bash"));
        let bash = process_queue_if_ready().expect("bash command");
        assert_eq!(bash.len(), 1);
        assert_eq!(bash[0].mode, "bash");

        clear_command_queue();
    }

    #[test]
    fn leaves_subagent_commands_for_their_consumer() {
        let _lock = TEST_QUEUE_LOCK.lock().unwrap();
        clear_command_queue();
        let mut command = QueuedCommand::new("worker", "task-notification");
        command.agent_id = Some("agent-1".to_string());
        enqueue(command);
        assert!(process_queue_if_ready().is_none());
        clear_command_queue();
    }

    #[test]
    fn skip_slash_commands_still_keep_slash_isolation() {
        let _lock = TEST_QUEUE_LOCK.lock().unwrap();
        clear_command_queue();
        let mut bridge = QueuedCommand::new("/plain-text", "prompt");
        bridge.skip_slash_commands = true;
        enqueue(bridge);
        enqueue(QueuedCommand::new("ordinary prompt", "prompt"));

        let group = process_queue_if_ready().expect("slash-shaped command");
        assert_eq!(group.len(), 1);
        assert_eq!(group[0].value, "/plain-text");
        clear_command_queue();
    }

    #[test]
    fn slash_shape_detection_does_not_consume_skip_slash_commands() {
        let mut command = QueuedCommand::new("/plain-text", "prompt");
        command.skip_slash_commands = true;
        assert!(is_slash_command(&command));
    }
}
