//! Maps to: CC `components/NotebookEditToolUseRejectedMessage.tsx`.

use crate::components::highlighted_code::HighlightedCode;
use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct NotebookEditToolUseRejectedMessageProps {
    pub notebook_path: String,
    pub cell_id: Option<String>,
    pub new_source: String,
    pub cell_type: Option<String>,
    pub edit_mode: Option<String>,
    pub verbose: bool,
}

/// Maps to: CC `edit_mode === 'delete' ? 'delete' : `${edit_mode} cell in``.
pub fn notebook_rejected_operation(edit_mode: Option<&str>) -> String {
    let edit_mode = edit_mode.unwrap_or("replace");
    if edit_mode == "delete" {
        "delete".to_string()
    } else {
        format!("{edit_mode} cell in")
    }
}

/// Maps to: CC `cell_type === 'markdown' ? 'file.md' : 'file.py'`.
pub fn notebook_rejected_preview_file_path(cell_type: Option<&str>) -> &'static str {
    if cell_type == Some("markdown") {
        "file.md"
    } else {
        "file.py"
    }
}

/// Maps to: CC `relative(getCwd(), notebook_path)` for in-workspace notebooks.
pub fn notebook_rejected_display_path(notebook_path: &str, verbose: bool) -> String {
    if verbose {
        notebook_path.to_string()
    } else {
        crate::components::permissions::file_permission_dialog::file_permission_relative_to_cwd(
            notebook_path,
        )
    }
}

/// Maps to: CC `components/NotebookEditToolUseRejectedMessage.tsx#NotebookEditToolUseRejectedMessage`.
#[component]
pub fn NotebookEditToolUseRejectedMessage(
    props: &NotebookEditToolUseRejectedMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let operation = notebook_rejected_operation(props.edit_mode.as_deref());
    let path = notebook_rejected_display_path(&props.notebook_path, props.verbose);
    // CC `NotebookEditToolUseRejectedMessage.tsx:35` interpolates `{cell_id}`
    // as a JSX hole: an absent id renders as nothing, not the literal
    // "undefined". (The tool-use header at `UI.tsx:51` is a template string,
    // where "@undefined" IS correct — do not unify the two.)
    let cell_id = props.cell_id.as_deref().unwrap_or("");
    let show_preview = props.edit_mode.as_deref().unwrap_or("replace") != "delete";

    element! {
        MessageResponse {
            View(flex_direction: FlexDirection::Column) {
                View(flex_direction: FlexDirection::Row) {
                    Text(content: format!("User rejected {operation} "), color: theme.subtle, wrap: TextWrap::NoWrap)
                    Text(content: path, color: theme.subtle, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                    Text(content: format!(" at cell {cell_id}"), color: theme.subtle, wrap: TextWrap::NoWrap)
                }
                #(if show_preview {
                    Some(element! {
                        View(margin_top: 1u32, flex_direction: FlexDirection::Column) {
                            HighlightedCode(
                                code: props.new_source.clone(),
                                file_path: notebook_rejected_preview_file_path(props.cell_type.as_deref()).to_string(),
                                dim: true,
                            )
                        }
                    })
                } else {
                    None
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn notebook_rejected_helpers_match_official_operation_and_language_branches() {
        assert_eq!(notebook_rejected_operation(None), "replace cell in");
        assert_eq!(
            notebook_rejected_operation(Some("insert")),
            "insert cell in"
        );
        assert_eq!(notebook_rejected_operation(Some("delete")), "delete");
        assert_eq!(
            notebook_rejected_preview_file_path(Some("markdown")),
            "file.md"
        );
        assert_eq!(notebook_rejected_preview_file_path(Some("code")), "file.py");
    }

    #[test]
    fn notebook_rejected_message_renders_preview_for_replace() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                NotebookEditToolUseRejectedMessage(
                    notebook_path: "/repo/notebooks/demo.ipynb".to_string(),
                    cell_id: Some("abc123".to_string()),
                    new_source: "print('hello')".to_string(),
                    cell_type: Some("code".to_string()),
                    edit_mode: Some("replace".to_string()),
                    verbose: true,
                )
            }
        }
        .render(Some(120))
        .to_string();

        assert!(
            text.contains(
                "User rejected replace cell in /repo/notebooks/demo.ipynb at cell abc123"
            ),
            "canvas=\n{text}"
        );
        assert!(text.contains("print('hello')"), "canvas=\n{text}");
    }

    #[test]
    fn notebook_rejected_message_hides_preview_for_delete() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                NotebookEditToolUseRejectedMessage(
                    notebook_path: "/repo/notebooks/demo.ipynb".to_string(),
                    cell_id: Some("abc123".to_string()),
                    new_source: "print('hello')".to_string(),
                    edit_mode: Some("delete".to_string()),
                    verbose: true,
                )
            }
        }
        .render(Some(120))
        .to_string();

        assert!(
            text.contains("User rejected delete /repo/notebooks/demo.ipynb at cell abc123"),
            "canvas=\n{text}"
        );
        assert!(!text.contains("print('hello')"), "canvas=\n{text}");
    }
}
