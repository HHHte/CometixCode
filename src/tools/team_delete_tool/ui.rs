//! UI-only renderer contract for official `TeamDeleteTool/UI.tsx`.

pub fn user_facing_name() -> &'static str {
    ""
}

pub fn render_tool_use_message() -> &'static str {
    "cleanup team: current"
}

pub fn renders_success_result(
    _success: bool,
    _team_name_present: bool,
    _message_present: bool,
) -> bool {
    // Official `renderToolResultMessage` always returns null for this tool;
    // batched teammate shutdown messages cover the visible UI.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_delete_keeps_empty_name_and_suppressed_success_contract() {
        assert_eq!(user_facing_name(), "");
        assert_eq!(render_tool_use_message(), "cleanup team: current");
        assert!(!renders_success_result(true, true, true));
        assert!(!renders_success_result(true, false, true));
    }
}
