//! Maps to: CC `components/FileEditToolUpdatedMessage.tsx`.
//!
//! Official success renderer for FileEdit/MultiEdit results. It summarizes the
//! structured patch and delegates detailed display to `StructuredDiffList`,
//! with a special preview-hint branch for plan files.

use crate::components::message_response::MessageResponse;
use crate::components::structured_diff_list::StructuredDiffList;
use crate::types::message::StructuredDiffHunk;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct FileEditToolUpdatedMessageProps {
    pub file_path: String,
    pub structured_patch: Vec<StructuredDiffHunk>,
    pub first_line: Option<String>,
    pub file_content: Option<String>,
    /// Official `style?: 'condensed'`; any value other than `condensed` is
    /// treated as the regular branch.
    pub style: Option<String>,
    pub verbose: bool,
    pub preview_hint: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEditUpdatedSummary {
    pub additions: usize,
    pub removals: usize,
    pub text: String,
}

/// Maps to: CC `structuredPatch.reduce(... startsWith('+/-'))` and summary JSX.
pub fn file_edit_updated_summary(
    structured_patch: &[StructuredDiffHunk],
) -> FileEditUpdatedSummary {
    let additions = structured_patch
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .filter(|line| line.starts_with('+'))
        .count();
    let removals = structured_patch
        .iter()
        .flat_map(|hunk| hunk.lines.iter())
        .filter(|line| line.starts_with('-'))
        .count();

    let mut text = String::new();
    if additions > 0 {
        text.push_str(&format!(
            "Added {additions} {}",
            if additions > 1 { "lines" } else { "line" }
        ));
    }
    if additions > 0 && removals > 0 {
        text.push_str(", ");
    }
    if removals > 0 {
        if additions == 0 {
            text.push_str("Removed");
        } else {
            text.push_str("removed");
        }
        text.push_str(&format!(
            " {removals} {}",
            if removals > 1 { "lines" } else { "line" }
        ));
    }

    FileEditUpdatedSummary {
        additions,
        removals,
        text,
    }
}

fn is_condensed(style: Option<&str>) -> bool {
    style == Some("condensed")
}

/// Maps to: CC `components/FileEditToolUpdatedMessage.tsx#FileEditToolUpdatedMessage`.
#[component]
pub fn FileEditToolUpdatedMessage(
    props: &FileEditToolUpdatedMessageProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let (columns, _) = hooks.use_terminal_size();
    let diff_width = columns.saturating_sub(12).max(1) as usize;
    let summary = file_edit_updated_summary(&props.structured_patch).text;

    let body = if let Some(preview_hint) = props.preview_hint.as_ref() {
        if !is_condensed(props.style.as_deref()) && !props.verbose {
            element! {
                MessageResponse {
                    Text(content: preview_hint.clone(), color: theme.inactive, dim: true, wrap: TextWrap::Wrap)
                }
            }
            .into_any()
        } else {
            element! {
                MessageResponse {
                    View(flex_direction: FlexDirection::Column) {
                        Text(content: summary, wrap: TextWrap::Wrap)
                        StructuredDiffList(
                            hunks: props.structured_patch.clone(),
                            dim: false,
                            width: diff_width,
                            file_path: props.file_path.clone(),
                            first_line: props.first_line.clone(),
                            file_content: props.file_content.clone(),
                        )
                    }
                }
            }
            .into_any()
        }
    } else if is_condensed(props.style.as_deref()) && !props.verbose {
        element! {
            Text(content: summary, wrap: TextWrap::Wrap)
        }
        .into_any()
    } else {
        element! {
            MessageResponse {
                View(flex_direction: FlexDirection::Column) {
                    Text(content: summary, wrap: TextWrap::Wrap)
                    StructuredDiffList(
                        hunks: props.structured_patch.clone(),
                        dim: false,
                        width: diff_width,
                        file_path: props.file_path.clone(),
                        first_line: props.first_line.clone(),
                        file_content: props.file_content.clone(),
                    )
                }
            }
        }
        .into_any()
    };
    let children = vec![body];

    element! {
        Fragment { #(children) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn hunk(lines: &[&str]) -> StructuredDiffHunk {
        StructuredDiffHunk {
            old_start: 1,
            old_lines: lines.iter().filter(|line| !line.starts_with('+')).count(),
            new_start: 1,
            new_lines: lines.iter().filter(|line| !line.starts_with('-')).count(),
            lines: lines.iter().map(|line| line.to_string()).collect(),
        }
    }

    #[test]
    fn file_edit_updated_summary_matches_official_pluralization() {
        let summary = file_edit_updated_summary(&[hunk(&["-old", "+new", "+extra"])]);
        assert_eq!(summary.additions, 2);
        assert_eq!(summary.removals, 1);
        assert_eq!(summary.text, "Added 2 lines, removed 1 line");

        let removed = file_edit_updated_summary(&[hunk(&["-old"])]);
        assert_eq!(removed.text, "Removed 1 line");
    }

    #[test]
    fn file_edit_updated_message_uses_preview_hint_regular_branch() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                FileEditToolUpdatedMessage(
                    file_path: "PLAN.md".to_string(),
                    structured_patch: vec![hunk(&["-old", "+new"])],
                    verbose: false,
                    preview_hint: Some("Run /plan to view the full plan".to_string()),
                )
            }
        }
        .render(Some(120))
        .to_string();

        assert!(
            text.contains("Run /plan to view the full plan"),
            "canvas=\n{text}"
        );
        assert!(!text.contains("Added 1 line"), "canvas=\n{text}");
    }

    #[test]
    fn file_edit_updated_message_renders_summary_and_diff() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                // Renders StructuredDiffList, which reads
                // `settings.syntax_highlighting_disabled` — strict since P7.
                crate::state::app_state::AppStateProvider(
                    children: crate::state::app_state::ProviderChildren::new(|| element! {
                        FileEditToolUpdatedMessage(
                            file_path: "src/lib.rs".to_string(),
                            structured_patch: vec![hunk(&["-old", "+new"])],
                            verbose: true,
                        )
                    }.into_any()),
                )
            }
        }
        .render(Some(120))
        .to_string();

        assert!(
            text.contains("Added 1 line, removed 1 line"),
            "canvas=\n{text}"
        );
        assert!(text.contains("-old"), "canvas=\n{text}");
        assert!(text.contains("+new"), "canvas=\n{text}");
    }
}
