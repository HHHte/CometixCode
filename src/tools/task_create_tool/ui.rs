//! UI-only renderer contract for official `TaskCreateTool`.

pub fn user_facing_name() -> &'static str {
    "TaskCreate"
}

pub fn render_tool_use_message() -> Option<&'static str> {
    None
}

pub fn renders_success_result() -> bool {
    false
}

pub fn map_tool_result_content(task_id: &str, subject: &str) -> String {
    format!("Task #{task_id} created successfully: {subject}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_create_matches_official_null_renderer_contract() {
        assert_eq!(user_facing_name(), "TaskCreate");
        assert_eq!(render_tool_use_message(), None);
        assert!(!renders_success_result());
        assert_eq!(
            map_tool_result_content("7", "Review auth"),
            "Task #7 created successfully: Review auth"
        );
    }
}
