//! Maps to: CC `components/tasks/AsyncAgentDetailDialog.tsx:1-200`.

use super::task_status_utils::{
    TaskStatus, TaskStatusOptions, get_task_status_color, get_task_status_icon,
};
use crate::components::design_system::dialog::Dialog;
use crate::components::messages::user_plan_message::UserPlanMessage;
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AsyncAgentDetailData {
    pub agent_type: Option<String>,
    pub description: String,
    pub prompt: String,
    pub status: TaskStatus,
    pub elapsed: String,
    pub token_count: Option<u64>,
    pub tool_use_count: Option<usize>,
    pub recent_activities: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Copy)]
enum Action {
    Done,
    Back,
    Stop,
}

#[derive(Default, Props)]
pub struct AsyncAgentDetailDialogProps<'a> {
    pub agent: Option<AsyncAgentDetailData>,
    pub on_done: HandlerMut<'a, ()>,
    pub on_kill_agent: HandlerMut<'a, ()>,
    pub on_back: HandlerMut<'a, ()>,
}

#[component]
pub fn AsyncAgentDetailDialog<'a>(
    props: &mut AsyncAgentDetailDialogProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let Some(agent) = props.agent.clone() else {
        return element! { Fragment }.into_any();
    };
    let mut pending = hooks.use_state(|| None::<Action>);
    let action = { *pending.read() };
    if let Some(action) = action {
        pending.set(None);
        match action {
            Action::Done => (props.on_done)(()),
            Action::Back if !props.on_back.is_default() => (props.on_back)(()),
            Action::Stop
                if agent.status == TaskStatus::Running && !props.on_kill_agent.is_default() =>
            {
                (props.on_kill_agent)(())
            }
            _ => {}
        }
    }
    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    for (name, context, action, active) in [
        ("confirm:yes", ContextName::Confirmation, Action::Done, true),
        ("task:close", ContextName::Task, Action::Done, true),
        (
            "task:back",
            ContextName::Task,
            Action::Back,
            !props.on_back.is_default(),
        ),
        (
            "task:stop",
            ContextName::Task,
            Action::Stop,
            agent.status == TaskStatus::Running && !props.on_kill_agent.is_default(),
        ),
    ] {
        let mut pending = pending;
        use_keybinding(
            &mut hooks,
            runtime.clone(),
            name,
            context,
            move || active,
            move || {
                pending.set(Some(action));
                true
            },
        );
    }
    let theme = hooks.use_context::<Theme>();
    let status = if agent.status == TaskStatus::Running {
        None
    } else {
        let label = match agent.status {
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Killed => "Stopped",
            TaskStatus::Pending => "Pending",
            TaskStatus::Running => "Running",
        };
        Some(format!(
            "{} {label} · ",
            get_task_status_icon(agent.status, TaskStatusOptions::default())
        ))
    };
    let subtitle = format!(
        "{}{}{}{}",
        status.unwrap_or_default(),
        agent.elapsed,
        agent
            .token_count
            .filter(|count| *count > 0)
            .map(|count| format!(" · {} tokens", crate::utils::format::format_number(count)))
            .unwrap_or_default(),
        agent
            .tool_use_count
            .filter(|count| *count > 0)
            .map(|count| format!(" · {count} {}", if count == 1 { "tool" } else { "tools" }))
            .unwrap_or_default()
    );
    let plan = crate::utils::messages::extract_tag(&agent.prompt, "plan");
    let display_prompt = if agent.prompt.chars().count() > 300 {
        format!("{}…", agent.prompt.chars().take(297).collect::<String>())
    } else {
        agent.prompt.clone()
    };
    let activities = agent.recent_activities.iter().enumerate().map(|(index, activity)| {
        let last = index + 1 == agent.recent_activities.len();
        element! { Text(content: format!("{} {activity}", if last { "›" } else { " " }), dim: !last, wrap: TextWrap::Truncate) }
    }).collect::<Vec<_>>();
    let guide = format!(
        "{}Esc/Enter/Space to close{}",
        if props.on_back.is_default() {
            ""
        } else {
            "← to go back · "
        },
        if agent.status == TaskStatus::Running && !props.on_kill_agent.is_default() {
            " · x to stop"
        } else {
            ""
        }
    );
    let done = pending;
    element! {
        Dialog(
            title: format!("{} › {}", agent.agent_type.as_deref().unwrap_or("agent"), if agent.description.is_empty() { "Async agent" } else { &agent.description }),
            subtitle: Some(subtitle), color: Some(theme.background), input_guide: Some(guide),
            on_cancel: move |_| { let mut done = done; done.set(Some(Action::Done)); },
        ) {
            View(flex_direction: FlexDirection::Column) {
                #((agent.status == TaskStatus::Running && !activities.is_empty()).then(|| element! { View(flex_direction: FlexDirection::Column) {
                    Text(content: "Progress".to_string(), weight: Weight::Bold, dim: true) #(activities)
                }}))
                #(plan.map(|content| element! { View(margin_top: 1u32) { UserPlanMessage(content: content, add_margin: false) } }).or_else(|| Some(element! { View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
                    Text(content: "Prompt".to_string(), weight: Weight::Bold, dim: true) Text(content: display_prompt)
                }})))
                #((agent.status == TaskStatus::Failed).then(|| agent.error.clone()).flatten().map(|error| element! { View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
                    Text(content: "Error".to_string(), weight: Weight::Bold, color: get_task_status_color(&theme, TaskStatus::Failed, TaskStatusOptions::default())) Text(content: error, color: theme.error)
                }}))
            }
        }
    }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn running_progress_and_failed_error_shapes_render() {
        let theme = *crate::utils::theme::current();
        let running = AsyncAgentDetailData {
            description: "review".to_string(),
            prompt: "do work".to_string(),
            status: TaskStatus::Running,
            elapsed: "12s".to_string(),
            recent_activities: vec!["Read(a.rs)".to_string(), "Edit(a.rs)".to_string()],
            ..Default::default()
        };
        let text = element! { ContextProvider(value: Context::owned(theme)) { AsyncAgentDetailDialog(agent: Some(running)) } }.render(Some(100)).to_string();
        assert!(text.contains("agent › review"));
        assert!(text.contains("Progress"));
        assert!(text.contains("› Edit(a.rs)"));
        let failed = AsyncAgentDetailData {
            prompt: "x".to_string(),
            status: TaskStatus::Failed,
            elapsed: "1s".to_string(),
            error: Some("boom".to_string()),
            ..Default::default()
        };
        let text = element! { ContextProvider(value: Context::owned(theme)) { AsyncAgentDetailDialog(agent: Some(failed)) } }.render(Some(100)).to_string();
        assert!(text.contains("Failed"));
        assert!(text.contains("Error"));
        assert!(text.contains("boom"));
    }
}
