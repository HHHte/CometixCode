//! Maps to: CC
//! `components/permissions/FileEditPermissionRequest/FileEditPermissionRequest.tsx`.
//!
//! Ports file-edit permission copy/path parsing and mounts the retained
//! `FileEditToolDiff` boundary. IDE modifications remain typed config/apply
//! callbacks for the dialog runtime.

use super::file_permission_dialog::{
    FileOperationType, FilePermissionDialog, FilePermissionOptionValue, IDEDiffConfig,
    IDEDiffFileEdit, create_single_edit_diff_config, file_permission_basename,
    file_permission_option_to_prompt_response, file_permission_relative_to_cwd,
};
use super::worker_badge::WorkerBadgeProps;
use crate::components::file_edit_tool_diff::{FileEdit, FileEditToolDiff};
use crate::tool::ToolPermissionContext;
use crate::types::permissions::{
    PermissionPromptChoice, PermissionPromptResponse, PermissionRequest as PermissionRequestData,
};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct FileEditPermissionRequestProps {
    pub request: Option<PermissionRequestData>,
    pub tool_permission_context: Option<ToolPermissionContext>,
    pub worker_badge: Option<WorkerBadgeProps>,
    pub on_select: Handler<PermissionPromptChoice>,
    pub on_select_response: Handler<PermissionPromptResponse>,
    pub on_cancel: Handler<()>,
}

fn default_request() -> PermissionRequestData {
    PermissionRequestData {
        permission_result: None,
        id: String::new(),
        tool_use_id: String::new(),
        tool_name: "Edit".to_string(),
        mcp_info: None,
        decision_reason: None,
        description: String::new(),
        message: String::new(),
        input_summary: String::new(),
        input: serde_json::Value::Null,
        call_input: None,
        rule: crate::types::permissions::PermissionRuleValue::new("Edit", None),
        suggestions: Vec::new(),
        blocked_path: None,
        metadata: None,
        is_compound_command: false,
        mode: crate::types::permissions::PermissionMode::Default,
    }
}

pub fn file_edit_permission_path(input: &serde_json::Value) -> String {
    input
        .get("file_path")
        .or_else(|| input.get("filePath"))
        .or_else(|| input.get("path"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub fn file_edit_permission_edit(input: &serde_json::Value) -> FileEdit {
    FileEdit {
        old_string: input
            .get("old_string")
            .or_else(|| input.get("oldString"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        new_string: input
            .get("new_string")
            .or_else(|| input.get("newString"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        replace_all: input
            .get("replace_all")
            .or_else(|| input.get("replaceAll"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    }
}

/// Maps to: CC `FileEditPermissionRequest.tsx#ideDiffSupport.getConfig`.
pub fn file_edit_ide_diff_config(input: &serde_json::Value) -> IDEDiffConfig {
    let edit = file_edit_permission_edit(input);
    create_single_edit_diff_config(
        file_edit_permission_path(input),
        edit.old_string,
        edit.new_string,
        Some(edit.replace_all),
    )
}

/// Maps to: CC `FileEditPermissionRequest.tsx#ideDiffSupport.applyChanges`.
pub fn apply_file_edit_ide_changes(
    input: &serde_json::Value,
    modified_edits: &[IDEDiffFileEdit],
) -> serde_json::Value {
    let mut updated = input.clone();
    let Some(first_edit) = modified_edits.first() else {
        return updated;
    };
    let object = updated
        .as_object_mut()
        .expect("file edit permission input is an object");
    object.insert(
        "old_string".to_string(),
        serde_json::Value::String(first_edit.old_string.clone()),
    );
    object.insert(
        "new_string".to_string(),
        serde_json::Value::String(first_edit.new_string.clone()),
    );
    object.insert(
        "replace_all".to_string(),
        serde_json::Value::Bool(first_edit.replace_all.unwrap_or(false)),
    );
    updated
}

/// Maps to: CC `FileEditPermissionRequest` render path.
#[component]
pub fn FileEditPermissionRequest(
    props: &mut FileEditPermissionRequestProps,
    _hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let request = props.request.clone().unwrap_or_else(default_request);
    let path = file_edit_permission_path(&request.input);
    let basename = file_permission_basename(&path);
    let subtitle = (!path.trim().is_empty()).then(|| file_permission_relative_to_cwd(&path));
    let question = format!("Do you want to make this edit to {basename}?");
    let edit = file_edit_permission_edit(&request.input);

    let tool_permission_context =
        props
            .tool_permission_context
            .clone()
            .unwrap_or_else(|| ToolPermissionContext {
                mode: request.mode,
                ..ToolPermissionContext::default()
            });
    let path_for_response = path.clone();
    let context_for_response = tool_permission_context.clone();
    let on_select = props.on_select.clone();
    let on_select_response = props.on_select_response.clone();
    let on_cancel = props.on_cancel.clone();

    element! {
        FilePermissionDialog(
            title: "Edit file".to_string(),
            subtitle: subtitle,
            question: Some(question),
            content_children: vec![element! {
                FileEditToolDiff(file_path: path.clone(), edits: vec![edit])
            }.into_any()],
            path: path,
            operation_type: FileOperationType::Write,
            tool_permission_context: Some(tool_permission_context),
            worker_badge: props.worker_badge.clone(),
            on_select: move |value: FilePermissionOptionValue| {
                let response = file_permission_option_to_prompt_response(
                    value,
                    &path_for_response,
                    FileOperationType::Write,
                    &context_for_response,
                );
                (on_select)(response.choice);
                (on_select_response)(response);
            },
            on_cancel: move |_| {
                (on_cancel)(());
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::{PermissionMode, PermissionRuleValue};
    use crate::utils::theme;

    fn edit_request() -> PermissionRequestData {
        PermissionRequestData {
            permission_result: None,
            id: "req".to_string(),
            tool_use_id: "toolu".to_string(),
            tool_name: "Edit".to_string(),
            mcp_info: None,
            decision_reason: None,
            description: String::new(),
            message: String::new(),
            input_summary: "/repo/src/lib.rs".to_string(),
            input: serde_json::json!({
                "file_path": "/repo/src/lib.rs",
                "old_string": "let old = true;",
                "new_string": "let new = true;"
            }),
            call_input: None,
            rule: PermissionRuleValue::new("Edit", Some("/repo/src/lib.rs".to_string())),
            suggestions: Vec::new(),
            blocked_path: None,
            metadata: None,
            is_compound_command: false,
            mode: PermissionMode::Default,
        }
    }

    #[test]
    fn file_edit_permission_helpers_match_official_prompt_copy() {
        let request = edit_request();
        assert_eq!(
            file_edit_permission_path(&request.input),
            "/repo/src/lib.rs"
        );
        let edit = file_edit_permission_edit(&request.input);
        assert_eq!(edit.old_string, "let old = true;");
        assert_eq!(edit.new_string, "let new = true;");
        assert!(!edit.replace_all);
    }

    #[test]
    fn file_edit_ide_diff_support_matches_official_get_config_and_apply_changes() {
        let request = edit_request();
        let config = file_edit_ide_diff_config(&request.input);
        assert_eq!(config.file_path, "/repo/src/lib.rs");
        assert_eq!(config.edits.len(), 1);
        assert_eq!(config.edits[0].old_string, "let old = true;");
        assert_eq!(config.edits[0].new_string, "let new = true;");
        assert_eq!(config.edits[0].replace_all, Some(false));

        let updated = apply_file_edit_ide_changes(
            &request.input,
            &[super::IDEDiffFileEdit {
                old_string: "before".to_string(),
                new_string: "after".to_string(),
                replace_all: Some(true),
            }],
        );
        assert_eq!(updated["old_string"], "before");
        assert_eq!(updated["new_string"], "after");
        assert_eq!(updated["replace_all"], true);
    }

    #[test]
    fn file_edit_permission_request_renders_official_title_and_question() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                FileEditPermissionRequest(request: Some(edit_request()))
            }
        }
        .render(Some(120))
        .to_string();
        assert!(text.contains("Edit file"), "canvas=\n{text}");
        assert!(
            text.contains("Do you want to make this edit to lib.rs?"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains('…')
                || (text.contains("let old = true") && text.contains("let new = true")),
            "canvas=\n{text}"
        );
    }
}
