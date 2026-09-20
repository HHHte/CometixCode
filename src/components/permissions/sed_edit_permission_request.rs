//! Maps to: CC
//! `components/permissions/SedEditPermissionRequest/SedEditPermissionRequest.tsx`.
//!
//! Ports the official Bash `sed -i` permission branch: read the target file,
//! compute the previewed substitution via `sedEditParser`, render it through
//! `FilePermissionDialog`, and attach the internal `_simulatedSedEdit` payload
//! to approved input so Bash execution writes exactly the previewed content.
//! Runtime-only IDE diff editing and async Suspense file loading remain outside
//! this retained-mode component; the current port performs a small synchronous
//! read during render like other file permission preview slices.

use super::file_permission_dialog::{
    FileOperationType, FilePermissionDialog, FilePermissionOptionValue, file_permission_basename,
    file_permission_option_to_prompt_response, file_permission_relative_to_cwd,
};
use super::worker_badge::WorkerBadgeProps;
use crate::tool::ToolPermissionContext;
use crate::tools::bash_tool::sed_edit_parser::{SedEditInfo, apply_sed_substitution};
use crate::types::permissions::{
    PermissionPromptChoice, PermissionPromptResponse, PermissionRequest as PermissionRequestData,
};
use crate::utils::diff::simple_diff_lines;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct SedEditPermissionRequestProps {
    pub request: Option<PermissionRequestData>,
    pub sed_info: Option<SedEditInfo>,
    pub tool_permission_context: Option<ToolPermissionContext>,
    pub worker_badge: Option<WorkerBadgeProps>,
    pub on_select: Handler<PermissionPromptResponse>,
    pub on_cancel: Handler<()>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SedFileReadResult {
    pub old_content: String,
    pub file_exists: bool,
}

fn default_request() -> PermissionRequestData {
    PermissionRequestData {
        permission_result: None,
        id: String::new(),
        tool_use_id: String::new(),
        tool_name: "Bash".to_string(),
        mcp_info: None,
        decision_reason: None,
        description: String::new(),
        message: String::new(),
        input_summary: String::new(),
        input: serde_json::Value::Null,
        call_input: None,
        rule: crate::types::permissions::PermissionRuleValue::new("Bash", None),
        suggestions: Vec::new(),
        blocked_path: None,
        metadata: None,
        is_compound_command: false,
        mode: crate::types::permissions::PermissionMode::Default,
    }
}

/// Maps to: CC `contentPromise` file-read branch.
pub fn read_sed_file_content(file_path: &str) -> SedFileReadResult {
    match std::fs::read_to_string(file_path) {
        Ok(raw) => SedFileReadResult {
            old_content: raw.replace("\r\n", "\n"),
            file_exists: true,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => SedFileReadResult {
            old_content: String::new(),
            file_exists: false,
        },
        Err(_) => SedFileReadResult {
            old_content: String::new(),
            file_exists: false,
        },
    }
}

/// Maps to: CC `noChangesMessage`.
pub fn sed_no_changes_message(file_exists: bool) -> &'static str {
    if file_exists {
        "Pattern did not match any content"
    } else {
        "File does not exist"
    }
}

/// Maps to: CC `edits` construction for `FileEditToolDiff`.
pub fn sed_edit_preview_content(old_content: &str, new_content: &str, file_exists: bool) -> String {
    if old_content == new_content {
        return sed_no_changes_message(file_exists).to_string();
    }
    simple_diff_lines(old_content, new_content).join("\n")
}

/// Maps to: CC `parseInput` adding `_simulatedSedEdit`.
pub fn sed_edit_updated_input(
    input: &serde_json::Value,
    file_path: &str,
    new_content: &str,
) -> serde_json::Value {
    let mut updated = input.clone();
    let payload = serde_json::json!({
        "filePath": file_path,
        "newContent": new_content,
    });

    if let Some(object) = updated.as_object_mut() {
        object.insert("_simulatedSedEdit".to_string(), payload);
        updated
    } else {
        serde_json::json!({ "_simulatedSedEdit": payload })
    }
}

fn response_for_file_option(
    value: FilePermissionOptionValue,
    updated_input: &serde_json::Value,
    file_path: &str,
    tool_permission_context: &ToolPermissionContext,
) -> PermissionPromptResponse {
    let mut response = file_permission_option_to_prompt_response(
        value,
        file_path,
        FileOperationType::Write,
        tool_permission_context,
    );
    if matches!(
        response.choice,
        PermissionPromptChoice::AllowOnce | PermissionPromptChoice::AlwaysAllow
    ) {
        response.updated_input = Some(updated_input.clone());
    }
    response
}

/// Maps to: CC `SedEditPermissionRequest` / `SedEditPermissionRequestInner`.
#[component]
pub fn SedEditPermissionRequest(
    props: &mut SedEditPermissionRequestProps,
    _hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let request = props.request.clone().unwrap_or_else(default_request);
    let sed_info = props.sed_info.clone().unwrap_or_else(|| SedEditInfo {
        file_path: String::new(),
        pattern: String::new(),
        replacement: String::new(),
        flags: String::new(),
        extended_regex: false,
    });
    let file_path = sed_info.file_path.clone();
    let read_result = read_sed_file_content(&file_path);
    let new_content = apply_sed_substitution(&read_result.old_content, &sed_info);
    let content = sed_edit_preview_content(
        &read_result.old_content,
        &new_content,
        read_result.file_exists,
    );
    let basename = file_permission_basename(&file_path);
    let subtitle =
        (!file_path.trim().is_empty()).then(|| file_permission_relative_to_cwd(&file_path));
    let question = format!("Do you want to make this edit to {basename}?");
    let updated_input = sed_edit_updated_input(&request.input, &file_path, &new_content);
    let tool_permission_context =
        props
            .tool_permission_context
            .clone()
            .unwrap_or_else(|| ToolPermissionContext {
                mode: request.mode,
                ..ToolPermissionContext::default()
            });
    let file_path_for_response = file_path.clone();
    let context_for_response = tool_permission_context.clone();
    let on_select = props.on_select.clone();
    let on_cancel = props.on_cancel.clone();

    element! {
        FilePermissionDialog(
            title: "Edit file".to_string(),
            subtitle: subtitle,
            question: Some(question),
            content: Some(content),
            path: file_path,
            operation_type: FileOperationType::Write,
            tool_permission_context: Some(tool_permission_context),
            worker_badge: props.worker_badge.clone(),
            on_select: move |value: FilePermissionOptionValue| {
                (on_select)(response_for_file_option(
                    value,
                    &updated_input,
                    &file_path_for_response,
                    &context_for_response,
                ));
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
    use crate::tools::bash_tool::sed_edit_parser::parse_sed_edit_command;
    use crate::types::permissions::{PermissionMode, PermissionRuleValue};
    use crate::utils::theme;

    fn sed_request(command: &str) -> PermissionRequestData {
        PermissionRequestData {
            permission_result: None,
            id: "req".to_string(),
            tool_use_id: "toolu".to_string(),
            tool_name: "Bash".to_string(),
            mcp_info: None,
            decision_reason: None,
            description: String::new(),
            message: String::new(),
            input_summary: command.to_string(),
            input: serde_json::json!({ "command": command }),
            call_input: None,
            rule: PermissionRuleValue::new("Bash", Some(command.to_string())),
            suggestions: Vec::new(),
            blocked_path: None,
            metadata: None,
            is_compound_command: false,
            mode: PermissionMode::Default,
        }
    }

    #[test]
    fn sed_edit_preview_content_and_updated_input_match_official_shape() {
        let input = serde_json::json!({ "command": "sed -i 's/old/new/' file.txt" });
        let updated = sed_edit_updated_input(&input, "file.txt", "new content");
        assert_eq!(
            updated.pointer("/_simulatedSedEdit/filePath"),
            Some(&serde_json::Value::String("file.txt".to_string()))
        );
        assert_eq!(
            updated.pointer("/_simulatedSedEdit/newContent"),
            Some(&serde_json::Value::String("new content".to_string()))
        );
        assert_eq!(
            sed_edit_preview_content("old\n", "new\n", true),
            "-old\n+new"
        );
        assert_eq!(
            sed_edit_preview_content("unchanged", "unchanged", true),
            "Pattern did not match any content"
        );
        assert_eq!(
            sed_edit_preview_content("", "", false),
            "File does not exist"
        );
    }

    #[test]
    fn sed_edit_permission_request_renders_file_edit_dialog_preview() {
        let path = std::env::temp_dir().join(format!(
            "cometix-sed-preview-{}.txt",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::write(&path, "hello old\n").unwrap();
        let command = format!("sed -i 's/old/new/' {}", path.display());
        let sed_info = parse_sed_edit_command(&command).unwrap();
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                SedEditPermissionRequest(
                    request: Some(sed_request(&command)),
                    sed_info: Some(sed_info),
                )
            }
        }
        .render(Some(140))
        .to_string();
        let _ = std::fs::remove_file(&path);

        assert!(text.contains("Edit file"), "canvas=\n{text}");
        assert!(
            text.contains("Do you want to make this edit"),
            "canvas=\n{text}"
        );
        assert!(text.contains("hello old"), "canvas=\n{text}");
        assert!(text.contains("hello new"), "canvas=\n{text}");
        assert!(text.contains("Yes"), "canvas=\n{text}");
        assert!(text.contains("No"), "canvas=\n{text}");
    }
}
