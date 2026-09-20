//! Maps to: CC
//! `components/permissions/NotebookEditPermissionRequest/NotebookEditPermissionRequest.tsx`.
//!
//! Ports notebook-edit permission copy/input parsing and mounts the rich
//! `NotebookEditToolDiff` boundary inside `FilePermissionDialog`.

use super::file_permission_dialog::{
    FileOperationType, FilePermissionDialog, FilePermissionOptionValue, file_permission_basename,
    file_permission_option_to_prompt_response,
};
use super::notebook_edit_tool_diff::{
    NotebookEditToolDiff, NotebookEditToolDiffInput, notebook_edit_tool_diff_text,
};
use super::worker_badge::WorkerBadgeProps;
use crate::tool::ToolPermissionContext;
use crate::types::permissions::{
    PermissionPromptChoice, PermissionPromptResponse, PermissionRequest as PermissionRequestData,
};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct NotebookEditPermissionRequestProps {
    pub request: Option<PermissionRequestData>,
    pub tool_permission_context: Option<ToolPermissionContext>,
    pub worker_badge: Option<WorkerBadgeProps>,
    pub verbose: bool,
    pub on_select: Handler<PermissionPromptChoice>,
    pub on_select_response: Handler<PermissionPromptResponse>,
    pub on_cancel: Handler<()>,
}

fn default_request() -> PermissionRequestData {
    PermissionRequestData {
        permission_result: None,
        id: String::new(),
        tool_use_id: String::new(),
        tool_name: "NotebookEdit".to_string(),
        mcp_info: None,
        decision_reason: None,
        description: String::new(),
        message: String::new(),
        input_summary: String::new(),
        input: serde_json::Value::Null,
        call_input: None,
        rule: crate::types::permissions::PermissionRuleValue::new("NotebookEdit", None),
        suggestions: Vec::new(),
        blocked_path: None,
        metadata: None,
        is_compound_command: false,
        mode: crate::types::permissions::PermissionMode::Default,
    }
}

pub fn notebook_edit_permission_path(input: &serde_json::Value) -> String {
    input
        .get("notebook_path")
        .or_else(|| input.get("notebookPath"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub fn notebook_edit_permission_mode(input: &serde_json::Value) -> String {
    input
        .get("edit_mode")
        .or_else(|| input.get("editMode"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("replace")
        .to_string()
}

pub fn notebook_edit_permission_question(input: &serde_json::Value) -> String {
    let notebook_path = notebook_edit_permission_path(input);
    let basename = file_permission_basename(&notebook_path);
    let edit_mode = notebook_edit_permission_mode(input);
    let edit_type_text = match edit_mode.as_str() {
        "insert" => "insert this cell into",
        "delete" => "delete this cell from",
        _ => "make this edit to",
    };
    format!("Do you want to {edit_type_text} {basename}?")
}

pub fn notebook_edit_permission_diff_input(
    input: &serde_json::Value,
    verbose: bool,
) -> NotebookEditToolDiffInput {
    let notebook_path = notebook_edit_permission_path(input);
    let cell_id = input
        .get("cell_id")
        .or_else(|| input.get("cellId"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let cell_type = input
        .get("cell_type")
        .or_else(|| input.get("cellType"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string);
    let edit_mode = notebook_edit_permission_mode(input);
    let new_source = input
        .get("new_source")
        .or_else(|| input.get("newSource"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    NotebookEditToolDiffInput {
        notebook_path,
        cell_id,
        new_source,
        cell_type,
        edit_mode,
        verbose,
        width: if verbose { 120 } else { 80 },
    }
}

pub fn notebook_edit_permission_preview(input: &serde_json::Value, verbose: bool) -> String {
    notebook_edit_tool_diff_text(&notebook_edit_permission_diff_input(input, verbose))
}

/// Maps to: CC `NotebookEditPermissionRequest` render path.
#[component]
pub fn NotebookEditPermissionRequest(
    props: &mut NotebookEditPermissionRequestProps,
    _hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let request = props.request.clone().unwrap_or_else(default_request);
    let path = notebook_edit_permission_path(&request.input);
    let question = notebook_edit_permission_question(&request.input);
    let diff_input = notebook_edit_permission_diff_input(&request.input, props.verbose);

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
            title: "Edit notebook".to_string(),
            question: Some(question),
            content_children: vec![element! {
                NotebookEditToolDiff(
                    notebook_path: diff_input.notebook_path,
                    cell_id: diff_input.cell_id,
                    new_source: diff_input.new_source,
                    cell_type: diff_input.cell_type,
                    edit_mode: Some(diff_input.edit_mode),
                    verbose: diff_input.verbose,
                    width: diff_input.width,
                )
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

    fn notebook_request(edit_mode: &str) -> PermissionRequestData {
        PermissionRequestData {
            permission_result: None,
            id: "req".to_string(),
            tool_use_id: "toolu".to_string(),
            tool_name: "NotebookEdit".to_string(),
            mcp_info: None,
            decision_reason: None,
            description: String::new(),
            message: String::new(),
            input_summary: "/repo/notebooks/demo.ipynb".to_string(),
            input: serde_json::json!({
                "notebook_path": "/repo/notebooks/demo.ipynb",
                "cell_id": "abc123",
                "new_source": "print('hello')",
                "cell_type": "code",
                "edit_mode": edit_mode
            }),
            call_input: None,
            rule: PermissionRuleValue::new(
                "NotebookEdit",
                Some("/repo/notebooks/demo.ipynb".to_string()),
            ),
            suggestions: Vec::new(),
            blocked_path: None,
            metadata: None,
            is_compound_command: false,
            mode: PermissionMode::Default,
        }
    }

    #[test]
    fn notebook_edit_permission_question_matches_official_edit_modes() {
        assert_eq!(
            notebook_edit_permission_question(&notebook_request("replace").input),
            "Do you want to make this edit to demo.ipynb?"
        );
        assert_eq!(
            notebook_edit_permission_question(&notebook_request("insert").input),
            "Do you want to insert this cell into demo.ipynb?"
        );
        assert_eq!(
            notebook_edit_permission_question(&notebook_request("delete").input),
            "Do you want to delete this cell from demo.ipynb?"
        );
    }

    #[test]
    fn notebook_edit_permission_request_renders_title_while_async_preview_is_pending() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                // The dialog renders NotebookEditToolDiff, which reads
                // `settings.syntax_highlighting_disabled`. Defaults are the
                // fixture: this asserts the title and the ABSENCE of the async
                // preview on the first frame — neither depends on that setting.
                crate::state::app_state::AppStateProvider(
                    children: crate::state::app_state::ProviderChildren::new(|| element! {
                        NotebookEditPermissionRequest(request: Some(notebook_request("replace")), verbose: true)
                    }.into_any()),
                )
            }
        }
        .render(Some(120))
        .to_string();
        assert!(text.contains("Edit notebook"), "canvas=\n{text}");
        assert!(
            text.contains("Do you want to make this edit to demo.ipynb?"),
            "canvas=\n{text}"
        );
        // Maps to CC Suspense fallback={null}: the filesystem-backed preview
        // is absent on the first retained frame rather than blocking render.
        assert!(!text.contains("Replace cell contents"), "canvas=\n{text}");
    }
}
