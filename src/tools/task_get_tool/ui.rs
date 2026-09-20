//! UI-only renderer contract for official `TaskGetTool`.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGetView {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub blocked_by: Vec<String>,
    pub blocks: Vec<String>,
}

pub fn user_facing_name() -> &'static str {
    "TaskGet"
}

pub fn render_tool_use_message() -> Option<&'static str> {
    None
}

pub fn renders_success_result() -> bool {
    false
}

pub fn map_tool_result_content(task: Option<&TaskGetView>) -> String {
    let Some(task) = task else {
        return "Task not found".to_string();
    };

    let mut lines = vec![
        format!("Task #{}: {}", task.id, task.subject),
        format!("Status: {}", task.status),
        format!("Description: {}", task.description),
    ];
    if !task.blocked_by.is_empty() {
        lines.push(format!(
            "Blocked by: {}",
            task.blocked_by
                .iter()
                .map(|id| format!("#{id}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !task.blocks.is_empty() {
        lines.push(format!(
            "Blocks: {}",
            task.blocks
                .iter()
                .map(|id| format!("#{id}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_get_matches_official_null_renderer_and_content_mapping() {
        assert_eq!(user_facing_name(), "TaskGet");
        assert_eq!(render_tool_use_message(), None);
        assert!(!renders_success_result());
        assert_eq!(map_tool_result_content(None), "Task not found");
        let content = map_tool_result_content(Some(&TaskGetView {
            id: "3".to_string(),
            subject: "Fix auth".to_string(),
            description: "Patch token refresh".to_string(),
            status: "pending".to_string(),
            blocked_by: vec!["1".to_string()],
            blocks: vec!["5".to_string()],
        }));
        assert!(content.contains("Task #3: Fix auth"));
        assert!(content.contains("Blocked by: #1"));
        assert!(content.contains("Blocks: #5"));
    }
}
