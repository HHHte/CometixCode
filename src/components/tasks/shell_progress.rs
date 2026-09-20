//! Maps to: CC `components/tasks/ShellProgress.tsx:1-58`.

use super::task_status_utils::TaskStatus;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct TaskStatusTextProps {
    pub status: TaskStatus,
    pub label: Option<String>,
    pub suffix: Option<String>,
}

#[component]
pub fn TaskStatusText(props: &TaskStatusTextProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let color = match props.status {
        TaskStatus::Completed => Some(theme.success),
        TaskStatus::Failed => Some(theme.error),
        TaskStatus::Killed => Some(theme.warning),
        TaskStatus::Pending | TaskStatus::Running => None,
    };
    element! {
        Text(
            content: format!(
                "({}{})",
                props.label.clone().unwrap_or_else(|| match props.status {
                    TaskStatus::Pending => "pending".to_string(),
                    TaskStatus::Running => "running".to_string(),
                    TaskStatus::Completed => "completed".to_string(),
                    TaskStatus::Failed => "failed".to_string(),
                    TaskStatus::Killed => "killed".to_string(),
                }),
                props.suffix.clone().unwrap_or_default(),
            ),
            color: color,
            dim: true,
            wrap: TextWrap::NoWrap,
        )
    }
}

#[derive(Default, Props)]
pub struct ShellProgressProps {
    pub status: TaskStatus,
}

#[component]
pub fn ShellProgress(props: &ShellProgressProps) -> impl Into<AnyElement<'static>> {
    let (status, label) = match props.status {
        TaskStatus::Completed => (TaskStatus::Completed, Some("done".to_string())),
        TaskStatus::Failed => (TaskStatus::Failed, Some("error".to_string())),
        TaskStatus::Killed => (TaskStatus::Killed, Some("stopped".to_string())),
        TaskStatus::Running | TaskStatus::Pending => (TaskStatus::Running, None),
    };
    element! { TaskStatusText(status: status, label: label) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_status_labels_colors_and_suffixes_match_official() {
        let theme = *crate::utils::theme::current();
        for (status, expected, color) in [
            (TaskStatus::Completed, "(done)", Some(theme.success)),
            (TaskStatus::Failed, "(error)", Some(theme.error)),
            (TaskStatus::Killed, "(stopped)", Some(theme.warning)),
            (TaskStatus::Pending, "(running)", None),
        ] {
            let canvas = element! {
                ContextProvider(value: Context::owned(theme)) {
                    ShellProgress(status: status)
                }
            }
            .render(Some(30));
            assert_eq!(canvas.to_string().trim_end(), expected);
            assert_eq!(canvas.resolved_text_style(0, 0).unwrap().color, color);
            assert!(canvas.resolved_text_style(0, 0).unwrap().dim);
        }

        let suffix = element! {
            ContextProvider(value: Context::owned(theme)) {
                TaskStatusText(
                    status: TaskStatus::Completed,
                    label: Some("done".to_string()),
                    suffix: Some(", unread".to_string()),
                )
            }
        }
        .render(Some(30))
        .to_string();
        assert_eq!(suffix.trim_end(), "(done, unread)");
    }
}
