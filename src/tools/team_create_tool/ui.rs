//! UI-only renderer contract for official `TeamCreateTool/UI.tsx`.

pub fn user_facing_name() -> &'static str {
    ""
}

pub fn render_tool_use_message(team_name: Option<&str>) -> String {
    format!("create team: {}", team_name.unwrap_or_default())
}

pub fn renders_success_result() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_create_keeps_empty_name_hidden_contract() {
        assert_eq!(user_facing_name(), "");
        assert_eq!(
            render_tool_use_message(Some("reviewers")),
            "create team: reviewers"
        );
        assert!(!renders_success_result());
    }
}
