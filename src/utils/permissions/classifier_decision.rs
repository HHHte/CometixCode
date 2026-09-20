//! Maps to: CC `utils/permissions/classifierDecision.ts`.
//!
//! Auto-mode classifier allowlist for tools that do not need a YOLO classifier
//! API call. The classifier itself is owned by `yolo_classifier`.

// Maps to: CC `classifierDecision.ts:22` — import the defining constant.
use super::yolo_classifier::YOLO_CLASSIFIER_TOOL_NAME;

/// Maps to: CC `isAutoModeAllowlistedTool(toolName)`.
pub fn is_auto_mode_allowlisted_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        crate::tools::file_read_tool::prompt::FILE_READ_TOOL_NAME
            | crate::tools::grep_tool::prompt::GREP_TOOL_NAME
            | crate::tools::glob_tool::prompt::GLOB_TOOL_NAME
            | crate::tools::lsp_tool::prompt::LSP_TOOL_NAME
            | crate::tools::tool_search_tool::prompt::TOOL_SEARCH_TOOL_NAME
            | crate::tools::list_mcp_resources_tool::prompt::LIST_MCP_RESOURCES_TOOL_NAME
            | crate::tools::read_mcp_resource_tool::prompt::READ_MCP_RESOURCE_TOOL_NAME
            | crate::tools::todo_write_tool::constants::TODO_WRITE_TOOL_NAME
            | crate::tools::task_create_tool::prompt::TASK_CREATE_TOOL_NAME
            | crate::tools::task_get_tool::prompt::TASK_GET_TOOL_NAME
            | crate::tools::task_update_tool::prompt::TASK_UPDATE_TOOL_NAME
            | crate::tools::task_list_tool::prompt::TASK_LIST_TOOL_NAME
            | crate::tools::task_stop_tool::prompt::TASK_STOP_TOOL_NAME
            | crate::tools::task_output_tool::constants::TASK_OUTPUT_TOOL_NAME
            | crate::tools::ask_user_question_tool::prompt::ASK_USER_QUESTION_TOOL_NAME
            | crate::tools::enter_plan_mode_tool::constants::ENTER_PLAN_MODE_TOOL_NAME
            | crate::tools::exit_plan_mode_tool::constants::EXIT_PLAN_MODE_TOOL_NAME
            | crate::tools::team_create_tool::prompt::TEAM_CREATE_TOOL_NAME
            | crate::tools::team_delete_tool::prompt::TEAM_DELETE_TOOL_NAME
            | crate::tools::send_message_tool::prompt::SEND_MESSAGE_TOOL_NAME
            | crate::tools::sleep_tool::prompt::SLEEP_TOOL_NAME
            | YOLO_CLASSIFIER_TOOL_NAME
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_mode_allowlist_matches_official_safe_tools() {
        assert!(is_auto_mode_allowlisted_tool("Read"));
        assert!(is_auto_mode_allowlisted_tool("Grep"));
        assert!(is_auto_mode_allowlisted_tool("ListMcpResourcesTool"));
        assert!(is_auto_mode_allowlisted_tool("ReadMcpResourceTool"));
        assert!(is_auto_mode_allowlisted_tool("TodoWrite"));
        assert!(is_auto_mode_allowlisted_tool("AskUserQuestion"));
        assert!(is_auto_mode_allowlisted_tool("TeamCreate"));
        assert!(is_auto_mode_allowlisted_tool("SendMessage"));
        assert!(is_auto_mode_allowlisted_tool("Sleep"));
        assert!(is_auto_mode_allowlisted_tool("classify_result"));
    }

    #[test]
    fn write_edit_and_shell_tools_are_not_auto_mode_allowlisted() {
        assert!(!is_auto_mode_allowlisted_tool("Write"));
        assert!(!is_auto_mode_allowlisted_tool("Edit"));
        assert!(!is_auto_mode_allowlisted_tool("NotebookEdit"));
        assert!(!is_auto_mode_allowlisted_tool("Bash"));
        assert!(!is_auto_mode_allowlisted_tool("PowerShell"));
        assert!(!is_auto_mode_allowlisted_tool("WebFetch"));
    }
}
