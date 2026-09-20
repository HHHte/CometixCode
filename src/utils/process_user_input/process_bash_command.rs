//! Bash-mode (`!`) submission normalization.
//!
//! Maps to: CC `utils/processUserInput/processBashCommand.tsx` — the third
//! member of the `processUserInput` family, alongside `processTextPrompt` and
//! `processSlashCommand`.
//!
//! Split of responsibility versus CC: upstream's function is `async` and owns
//! shell execution itself (`BashTool.call(...)`), because `processUserInput`
//! can await. The iocraft submit handler cannot, so the REPL keeps running the
//! command on its own thread and hands the finished [`BashCommandOutcome`]
//! here. What lives in this file is the part CC's file actually defines: the
//! message shapes each outcome produces, and `shouldQuery: false` on all of
//! them.

use super::ProcessUserInputBaseResult;
use crate::constants::query_source::QuerySource;
use crate::types::message::{RenderableMessage, RenderableMessageKind, UserMessage};
use crate::utils::xml::escape_xml;

/// What the shell did, mapped onto CC's three `processBashCommand` exits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BashCommandOutcome {
    /// CC's `try` path: the tool returned data.
    Completed { stdout: String, stderr: String },
    /// CC `catch (e) { if (e instanceof ShellError) { if (e.interrupted)`
    /// (`processBashCommand.tsx:168`) — Ctrl-C or a timeout killed it.
    Interrupted,
    /// CC's final `catch` (`:190-202`), which renders `Command failed: …`.
    ///
    /// CC has a fourth exit — a non-interrupted `ShellError`, which still
    /// carries `stdout`/`stderr` (`:179-189`). Rust's `bash_output` returns
    /// `Result<BashOutput, String>`, so a failure has no output halves to
    /// report and collapses into this one.
    Failed { message: String },
}

fn user_row(content: String) -> RenderableMessage {
    RenderableMessage::user(uuid::Uuid::new_v4().to_string(), content)
}

fn row_of(message: UserMessage) -> RenderableMessage {
    RenderableMessage {
        uuid: message.uuid.clone(),
        kind: RenderableMessageKind::User { message },
    }
}

/// Maps to: CC `processBashCommand(...)` (`processBashCommand.tsx:27-205`).
///
/// Every exit is `[caveat, userMessage, …]` with `shouldQuery: false`: a `!`
/// command is something the user ran on the side, so it never starts a turn,
/// and the caveat tells the model not to answer the output as if it had been
/// addressed to it (see `create_synthetic_user_caveat_message`).
pub fn process_bash_command(
    command: &str,
    outcome: BashCommandOutcome,
    attachment_messages: Vec<RenderableMessage>,
) -> ProcessUserInputBaseResult {
    use crate::constants::xml::{BASH_INPUT_TAG, BASH_STDERR_TAG, BASH_STDOUT_TAG};

    let mut messages = vec![
        row_of(crate::utils::messages::create_synthetic_user_caveat_message()),
        // CC `createUserMessage({ content: prepareUserContent({ inputString:
        // `<bash-input>${inputString}</bash-input>`, precedingInputBlocks }) })`
        // (:46-51). `precedingInputBlocks` — pasted blocks that preceded the
        // command — has no Rust caller yet; it would prepend blocks here.
        user_row(format!("<{BASH_INPUT_TAG}>{command}</{BASH_INPUT_TAG}>")),
    ];

    match outcome {
        // CC :168-178. Note the ordering: the interruption marker comes BEFORE
        // the attachments here, where the other exits put attachments first.
        // And there is no bash-stdout/bash-stderr row at all — an interrupted
        // command reports the interrupt, not partial output.
        BashCommandOutcome::Interrupted => {
            messages.push(row_of(
                crate::utils::messages::create_user_interruption_message(false),
            ));
            messages.extend(attachment_messages);
        }
        BashCommandOutcome::Completed { stdout, stderr } => {
            messages.extend(attachment_messages);
            // Both tags ride ONE message (:155-160). `stdout` is NOT escaped:
            // CC passes the `processToolResultBlock` output through verbatim
            // because it may contain a trusted `<persisted-output>` wrapper,
            // and escaping would turn that structural tag into text the model
            // can no longer parse (:145-151). `stderr` is escaped.
            messages.push(user_row(format!(
                "<{BASH_STDOUT_TAG}>{stdout}</{BASH_STDOUT_TAG}><{BASH_STDERR_TAG}>{}</{BASH_STDERR_TAG}>",
                escape_xml(&stderr)
            )));
        }
        // CC :190-202 — stderr only, prefixed with `Command failed: `.
        BashCommandOutcome::Failed { message } => {
            messages.extend(attachment_messages);
            messages.push(user_row(format!(
                "<{BASH_STDERR_TAG}>Command failed: {}</{BASH_STDERR_TAG}>",
                escape_xml(&message)
            )));
        }
    }

    ProcessUserInputBaseResult {
        messages,
        should_query: false,
        allowed_tools: None,
        local_action: None,
        query_source: QuerySource::Prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::UserContent;

    fn text_of(row: &RenderableMessage) -> &str {
        match &row.kind {
            RenderableMessageKind::User { message } => match message.first_content_block() {
                Some(UserContent::Text(text)) | Some(UserContent::MetaText(text)) => text,
                other => panic!("expected a text block, got {other:?}"),
            },
            other => panic!("expected a user row, got {other:?}"),
        }
    }

    /// Maps to: CC `processBashCommand.tsx:155-161`.
    #[test]
    fn completed_command_reports_both_streams_in_one_message() {
        let result = process_bash_command(
            "ls",
            BashCommandOutcome::Completed {
                stdout: "a\nb".to_string(),
                stderr: "warn <tag>".to_string(),
            },
            Vec::new(),
        );

        assert!(!result.should_query, "a `!` command never starts a turn");
        assert_eq!(result.messages.len(), 3);
        assert!(text_of(&result.messages[0]).starts_with("<local-command-caveat>"));
        assert_eq!(text_of(&result.messages[1]), "<bash-input>ls</bash-input>");
        // One message, both tags; stderr escaped, stdout verbatim.
        assert_eq!(
            text_of(&result.messages[2]),
            "<bash-stdout>a\nb</bash-stdout><bash-stderr>warn &lt;tag&gt;</bash-stderr>"
        );
    }

    /// Maps to: CC `processBashCommand.tsx:168-178` — the interrupt marker
    /// replaces the output row entirely, and lands before the attachments.
    #[test]
    fn interrupted_command_reports_the_interrupt_instead_of_output() {
        let result = process_bash_command("sleep 100", BashCommandOutcome::Interrupted, Vec::new());

        assert!(!result.should_query);
        assert_eq!(result.messages.len(), 3);
        assert_eq!(
            text_of(&result.messages[1]),
            "<bash-input>sleep 100</bash-input>"
        );
        assert_eq!(
            text_of(&result.messages[2]),
            crate::utils::messages::INTERRUPT_MESSAGE
        );
        assert!(
            !result
                .messages
                .iter()
                .any(|row| text_of(row).contains("<bash-stdout>")),
            "no output row for an interrupted command"
        );
    }

    /// Maps to: CC `processBashCommand.tsx:190-202`.
    #[test]
    fn failed_command_reports_stderr_only() {
        let result = process_bash_command(
            "nope",
            BashCommandOutcome::Failed {
                message: "no such file & directory".to_string(),
            },
            Vec::new(),
        );

        assert_eq!(
            text_of(&result.messages[2]),
            "<bash-stderr>Command failed: no such file &amp; directory</bash-stderr>"
        );
    }

    /// CC threads `attachmentMessages` into every exit (`:157`, `:174`).
    #[test]
    fn attachments_ride_along() {
        let attachment = RenderableMessage::user("att-1", "attached");
        let result = process_bash_command(
            "ls",
            BashCommandOutcome::Completed {
                stdout: String::new(),
                stderr: String::new(),
            },
            vec![attachment],
        );

        assert_eq!(result.messages.len(), 4);
        assert_eq!(text_of(&result.messages[2]), "attached");
    }
}
