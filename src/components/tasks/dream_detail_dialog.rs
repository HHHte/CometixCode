//! Maps to: CC `components/tasks/DreamDetailDialog.tsx:1-136`.

use super::task_status_utils::TaskStatus;
use crate::components::design_system::dialog::Dialog;
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

pub const VISIBLE_TURNS: usize = 6;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DreamTurn {
    pub text: String,
    pub tool_use_count: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DreamDetailData {
    pub status: TaskStatus,
    pub elapsed: String,
    pub sessions_reviewing: usize,
    pub files_touched: usize,
    pub turns: Vec<DreamTurn>,
}
#[derive(Clone, Copy)]
enum Action {
    Done,
    Back,
    Stop,
}

#[derive(Default, Props)]
pub struct DreamDetailDialogProps<'a> {
    pub task: Option<DreamDetailData>,
    pub on_done: HandlerMut<'a, ()>,
    pub on_back: HandlerMut<'a, ()>,
    pub on_kill: HandlerMut<'a, ()>,
}

#[component]
pub fn DreamDetailDialog<'a>(
    props: &mut DreamDetailDialogProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let Some(task) = props.task.clone() else {
        return element! { Fragment }.into_any();
    };
    let mut pending = hooks.use_state(|| None::<Action>);
    let action = { *pending.read() };
    if let Some(action) = action {
        pending.set(None);
        match action {
            Action::Done => (props.on_done)(()),
            Action::Back if !props.on_back.is_default() => (props.on_back)(()),
            Action::Stop if task.status == TaskStatus::Running && !props.on_kill.is_default() => {
                (props.on_kill)(())
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
            task.status == TaskStatus::Running && !props.on_kill.is_default(),
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
    let visible = task
        .turns
        .iter()
        .filter(|turn| !turn.text.is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let hidden = visible.len().saturating_sub(VISIBLE_TURNS);
    let shown = visible.into_iter().skip(hidden).map(|turn| element! { View(flex_direction: FlexDirection::Column) {
        Text(content: turn.text) #((turn.tool_use_count > 0).then(|| element! { Text(content: format!("  ({} {})", turn.tool_use_count, if turn.tool_use_count == 1 { "tool" } else { "tools" }), dim: true) }))
    }}.into_any()).collect::<Vec<_>>();
    let turn_rows: Vec<AnyElement<'static>> = if shown.is_empty() {
        vec![element! { Text(content: if task.status == TaskStatus::Running { "Starting…".to_string() } else { "(no text output)".to_string() }, dim: true) }.into_any()]
    } else {
        shown
    };
    let theme = hooks.use_context::<Theme>();
    let status_color = match task.status {
        TaskStatus::Running | TaskStatus::Pending => theme.background,
        TaskStatus::Completed => theme.success,
        TaskStatus::Failed | TaskStatus::Killed => theme.error,
    };
    let subtitle = format!(
        "{} · reviewing {} {}{}",
        task.elapsed,
        task.sessions_reviewing,
        if task.sessions_reviewing == 1 {
            "session"
        } else {
            "sessions"
        },
        if task.files_touched > 0 {
            format!(
                " · {} {} touched",
                task.files_touched,
                if task.files_touched == 1 {
                    "file"
                } else {
                    "files"
                }
            )
        } else {
            String::new()
        }
    );
    let guide = format!(
        "{}Esc/Enter/Space to close{}",
        if props.on_back.is_default() {
            ""
        } else {
            "← to go back · "
        },
        if task.status == TaskStatus::Running && !props.on_kill.is_default() {
            " · x to stop"
        } else {
            ""
        }
    );
    let mut done = pending;
    element! { Dialog(title: "Memory consolidation".to_string(), subtitle: Some(subtitle), color: Some(theme.background), input_guide: Some(guide), on_cancel: move |_| done.set(Some(Action::Done))) {
        View(flex_direction: FlexDirection::Column, row_gap: 1u32) {
            MixedText(contents: vec![MixedTextContent::new("Status: ").weight(Weight::Bold), MixedTextContent::new(match task.status { TaskStatus::Running => "running", TaskStatus::Pending => "pending", TaskStatus::Completed => "completed", TaskStatus::Failed => "failed", TaskStatus::Killed => "killed" }).color(status_color)])
            #((hidden > 0).then(|| element! { Text(content: format!("({hidden} earlier {})", if hidden == 1 { "turn" } else { "turns" }), dim: true) }))
            #(turn_rows)
        }
    }}.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filters_tool_only_turns_and_collapses_before_last_six() {
        let turns = (0..8)
            .map(|i| DreamTurn {
                text: format!("turn {i}"),
                tool_use_count: i,
            })
            .chain(std::iter::once(DreamTurn {
                text: String::new(),
                tool_use_count: 9,
            }))
            .collect();
        let data = DreamDetailData {
            status: TaskStatus::Running,
            elapsed: "5s".to_string(),
            sessions_reviewing: 2,
            files_touched: 1,
            turns,
        };
        let text = element! { ContextProvider(value: Context::owned(*crate::utils::theme::current())) { DreamDetailDialog(task: Some(data)) } }.render(Some(100)).to_string();
        assert!(text.contains("(2 earlier turns)"));
        assert!(!text.contains("turn 0"));
        assert!(text.contains("turn 7"));
        assert!(!text.contains("(9 tools)"));
    }
}
