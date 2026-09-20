//! Maps to: CC `components/PromptInput/PromptInputQueuedCommands.tsx:1-159`.

use crate::components::messages::highlighted_thinking_text::QueuedMessageContext;
use crate::components::messages::user_bash_input_message::UserBashInputMessage;
use crate::components::messages::user_text_message::UserTextMessage;
use crate::constants::xml::{STATUS_TAG, SUMMARY_TAG, TASK_NOTIFICATION_TAG};
use crate::utils::message_queue_manager::{QueuedCommand, is_queued_command_visible};
use iocraft::prelude::*;

pub const MAX_VISIBLE_NOTIFICATIONS: usize = 3;

pub fn is_idle_notification(value: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(value)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(|kind| kind.as_str())
                .map(str::to_string)
        })
        .as_deref()
        == Some("idle_notification")
}

pub fn create_overflow_notification_message(count: usize) -> String {
    format!(
        "<{TASK_NOTIFICATION_TAG}>\n<{SUMMARY_TAG}>+{count} more tasks completed</{SUMMARY_TAG}>\n<{STATUS_TAG}>completed</{STATUS_TAG}>\n</{TASK_NOTIFICATION_TAG}>"
    )
}

pub fn process_queued_commands(commands: &[QueuedCommand]) -> Vec<QueuedCommand> {
    let filtered = commands
        .iter()
        .filter(|command| !is_idle_notification(&command.value))
        .cloned()
        .collect::<Vec<_>>();
    let tasks = filtered
        .iter()
        .filter(|command| command.mode == "task-notification")
        .cloned()
        .collect::<Vec<_>>();
    let mut others = filtered
        .into_iter()
        .filter(|command| command.mode != "task-notification")
        .collect::<Vec<_>>();
    if tasks.len() <= MAX_VISIBLE_NOTIFICATIONS {
        others.extend(tasks);
        return others;
    }
    others.extend(tasks.into_iter().take(MAX_VISIBLE_NOTIFICATIONS - 1));
    others.push(QueuedCommand::new(
        create_overflow_notification_message(
            commands
                .iter()
                .filter(|command| {
                    command.mode == "task-notification" && !is_idle_notification(&command.value)
                })
                .count()
                - (MAX_VISIBLE_NOTIFICATIONS - 1),
        ),
        "task-notification",
    ));
    others
}

#[derive(Default, Props)]
pub struct PromptInputQueuedCommandsProps {
    pub commands: Option<Vec<QueuedCommand>>,
    pub viewing_agent: bool,
    pub use_brief_layout: bool,
}

#[component]
pub fn PromptInputQueuedCommands(
    props: &PromptInputQueuedCommandsProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let queue_snapshot = crate::hooks::use_command_queue::use_command_queue(&mut hooks);
    if props.viewing_agent {
        return element! { Fragment }.into_any();
    }
    let commands = props.commands.clone().unwrap_or(queue_snapshot);
    let visible = commands
        .into_iter()
        .filter(is_queued_command_visible)
        .collect::<Vec<_>>();
    if visible.is_empty() {
        return element! { Fragment }.into_any();
    }
    let rows = process_queued_commands(&visible).into_iter().map(|command| {
        let context = QueuedMessageContext { is_queued: true };
        if command.mode == "bash" {
            element! { ContextProvider(value: Context::owned(context)) { UserBashInputMessage(command: command.value, add_margin: false) } }.into_any()
        } else {
            element! { ContextProvider(value: Context::owned(context)) { UserTextMessage(content: command.value, add_margin: false, use_brief_layout: props.use_brief_layout, is_transcript_mode: false) } }.into_any()
        }
    }).collect::<Vec<_>>();
    element! { View(margin_top: 1u32, flex_direction: FlexDirection::Column) { #(rows) } }
        .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn idle_notifications_are_hidden_and_task_overflow_is_capped() {
        let mut commands = vec![
            QueuedCommand::new(r#"{"type":"idle_notification"}"#, "prompt"),
            QueuedCommand::new("normal", "prompt"),
        ];
        for index in 0..5 {
            commands.push(QueuedCommand::new(
                format!("task {index}"),
                "task-notification",
            ));
        }
        let result = process_queued_commands(&commands);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].value, "normal");
        assert_eq!(result[1].value, "task 0");
        assert_eq!(result[2].value, "task 1");
        assert!(result[3].value.contains("+3 more tasks completed"));
    }
    #[test]
    fn render_hides_agent_queue_and_wraps_bash_copy() {
        let bash = vec![QueuedCommand::new("echo hi", "bash")];
        let theme = *crate::utils::theme::current();
        let hidden = element! { ContextProvider(value: Context::owned(theme)) { PromptInputQueuedCommands(commands: Some(bash.clone()), viewing_agent: true) } }.render(Some(60)).to_string();
        assert!(hidden.trim().is_empty());
        let visible = element! { ContextProvider(value: Context::owned(theme)) { PromptInputQueuedCommands(commands: Some(bash)) } }.render(Some(60)).to_string();
        assert!(visible.contains("! echo hi"));
    }

    #[component]
    fn ReactiveQueueHarness(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
        let mut app = hooks.use_app();
        hooks.use_future(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
            crate::utils::message_queue_manager::enqueue(QueuedCommand::new(
                "reactive command",
                "prompt",
            ));
            futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
            app.exit();
        });
        element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                PromptInputQueuedCommands
            }
        }
    }

    #[test]
    fn live_component_re_renders_from_queue_subscription() {
        use futures::StreamExt;
        let _guard = crate::utils::message_queue_manager::TEST_QUEUE_LOCK
            .lock()
            .unwrap();
        crate::utils::message_queue_manager::clear_command_queue();
        let frames = futures::executor::block_on(
            element!(ReactiveQueueHarness)
                .mock_terminal_render_loop(MockTerminalConfig::default())
                .collect::<Vec<_>>(),
        );
        crate::utils::message_queue_manager::clear_command_queue();
        assert!(
            frames
                .iter()
                .any(|frame| frame.to_string().contains("reactive command"))
        );
    }
}
