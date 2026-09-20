//! Maps to: CC
//! `components/permissions/FileWritePermissionRequest/FileWriteToolDiff.tsx`.
//!
//! Renders syntax-highlighted new-file content or the retained
//! `StructuredDiff` generated for an overwrite, inside the official dashed
//! horizontal frame.

use crate::components::highlighted_code::HighlightedCode;
use crate::components::structured_diff_list::StructuredDiffList;
use iocraft::prelude::*;
use similar::{ChangeTag, TextDiff};

#[derive(Default, Props)]
pub struct FileWriteToolDiffProps {
    pub file_path: String,
    pub content: String,
    pub file_exists: bool,
    pub old_content: String,
}

/// Maps to: CC `FileWriteToolDiff` render branch.
#[component]
pub fn FileWriteToolDiff(
    props: &FileWriteToolDiffProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let (columns, _) = hooks.use_terminal_size();
    let width = (columns as usize).saturating_sub(2).max(1);
    let syntax_highlighting_disabled =
        crate::state::app_state::use_app_state(&mut hooks, |state| {
            state.settings.syntax_highlighting_disabled.unwrap_or(false)
        });
    let body: AnyElement<'static> = if props.file_exists {
        element! {
            StructuredDiffList(
                hunks: crate::utils::diff::structured_diff_hunks(&props.old_content, &props.content, 3),
                dim: false,
                width: width,
                file_path: props.file_path.clone(),
                first_line: props.content.lines().next().map(str::to_string),
                file_content: Some(props.old_content.clone()),
                skip_highlighting: syntax_highlighting_disabled,
            )
        }
        .into_any()
    } else {
        element! {
            HighlightedCode(
                code: if props.content.is_empty() { "(No content)".to_string() } else { props.content.clone() },
                file_path: props.file_path.clone(),
                width: Some(width),
                syntax_highlighting_disabled: syntax_highlighting_disabled,
            )
        }
        .into_any()
    };
    element! {
        View(flex_direction: FlexDirection::Column) {
            View(
                border_style: BorderStyle::Dashed,
                border_left: false,
                border_right: false,
                padding_left: 1u32,
                padding_right: 1u32,
                flex_direction: FlexDirection::Column,
            ) {
                #(body)
            }
        }
    }
}

/// Maps to: CC `FileWriteToolDiff` `hunks ? <StructuredDiff/> : <HighlightedCode/>`.
pub fn file_write_tool_diff_text(
    _file_path: &str,
    content: &str,
    file_exists: bool,
    old_content: &str,
) -> String {
    if !file_exists {
        return if content.is_empty() {
            "(No content)".to_string()
        } else {
            content.to_string()
        };
    }

    let diff = TextDiff::from_lines(old_content, content);
    let mut lines = Vec::new();
    for change in diff.iter_all_changes() {
        let prefix = match change.tag() {
            ChangeTag::Delete => '-',
            ChangeTag::Insert => '+',
            ChangeTag::Equal => ' ',
        };
        let value = change.value().trim_end_matches('\n');
        if value.is_empty() && change.value().contains('\n') {
            lines.push(prefix.to_string());
        } else {
            lines.push(format!("{prefix}{value}"));
        }
    }

    if lines.is_empty() {
        "(No changes)".to_string()
    } else {
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn file_write_tool_diff_renders_new_file_content_like_highlighted_code_branch() {
        assert_eq!(
            file_write_tool_diff_text("/tmp/new.txt", "hello", false, ""),
            "hello"
        );
        assert_eq!(
            file_write_tool_diff_text("/tmp/new.txt", "", false, ""),
            "(No content)"
        );
    }

    #[test]
    fn file_write_tool_diff_renders_overwrite_line_diff_like_structured_diff_branch() {
        let diff = file_write_tool_diff_text("/tmp/demo.txt", "alpha\nnew\n", true, "alpha\nold\n");
        assert!(diff.contains(" alpha"), "diff=\n{diff}");
        assert!(diff.contains("-old"), "diff=\n{diff}");
        assert!(diff.contains("+new"), "diff=\n{diff}");
    }

    #[test]
    fn file_write_tool_diff_component_renders_diff_text() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                // Reads `settings.syntax_highlighting_disabled`. Default state
                // is the fixture: the assertion is on the diff markers, which
                // are emitted either way.
                crate::state::app_state::AppStateProvider(
                    children: crate::state::app_state::ProviderChildren::new(|| element! {
                        FileWriteToolDiff(
                            file_path: "/tmp/demo.txt".to_string(),
                            content: "new\n".to_string(),
                            file_exists: true,
                            old_content: "old\n".to_string(),
                        )
                    }.into_any()),
                )
            }
        }
        .render(Some(80))
        .to_string();
        assert!(text.contains("-old"), "canvas=\n{text}");
        assert!(text.contains("+new"), "canvas=\n{text}");
    }
}
