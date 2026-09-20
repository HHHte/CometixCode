//! UI-only renderer contract for official `TaskListTool`.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskListItemView {
    pub id: String,
    pub subject: String,
    pub status: String,
    pub owner: Option<String>,
    pub blocked_by: Vec<String>,
}

pub fn user_facing_name() -> &'static str {
    "TaskList"
}

pub fn render_tool_use_message() -> Option<&'static str> {
    None
}

pub fn renders_success_result() -> bool {
    false
}

pub fn map_tool_result_content(tasks: &[TaskListItemView]) -> String {
    if tasks.is_empty() {
        return "No tasks found".to_string();
    }

    tasks
        .iter()
        .map(|task| {
            let owner = task
                .owner
                .as_deref()
                .filter(|owner| !owner.is_empty())
                .map(|owner| format!(" ({owner})"))
                .unwrap_or_default();
            let blocked = if task.blocked_by.is_empty() {
                String::new()
            } else {
                format!(
                    " [blocked by {}]",
                    task.blocked_by
                        .iter()
                        .map(|id| format!("#{id}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!(
                "#{} [{}] {}{}{}",
                task.id, task.status, task.subject, owner, blocked
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_list_matches_official_null_renderer_and_content_mapping() {
        assert_eq!(user_facing_name(), "TaskList");
        assert_eq!(render_tool_use_message(), None);
        assert!(!renders_success_result());
        assert_eq!(map_tool_result_content(&[]), "No tasks found");
        assert_eq!(
            map_tool_result_content(&[TaskListItemView {
                id: "2".to_string(),
                subject: "Run tests".to_string(),
                status: "in_progress".to_string(),
                owner: Some("runner".to_string()),
                blocked_by: vec!["1".to_string(), "4".to_string()],
            }]),
            "#2 [in_progress] Run tests (runner) [blocked by #1, #4]"
        );
    }
}
