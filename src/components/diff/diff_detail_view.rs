//! Maps to: CC `components/diff/DiffDetailView.tsx:1-140`.

use crate::components::design_system::divider::Divider;
use crate::components::structured_diff::StructuredDiff;
use crate::types::message::StructuredDiffHunk;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::path::PathBuf;

#[derive(Default, Props)]
pub struct DiffDetailViewProps {
    pub file_path: String,
    pub hunks: Vec<StructuredDiffHunk>,
    pub is_large_file: bool,
    pub is_binary: bool,
    pub is_truncated: bool,
    pub is_untracked: bool,
}

fn read_syntax_context(file_path: &str) -> (Option<String>, Option<String>) {
    if file_path.is_empty() {
        return (None, None);
    }
    let path = PathBuf::from(file_path);
    let full_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let content = std::fs::read_to_string(full_path).ok();
    let first_line = content
        .as_deref()
        .and_then(|content| content.split('\n').next())
        .map(str::to_string);
    (first_line, content)
}

fn special_file_detail(
    file_path: String,
    suffix: Option<&'static str>,
    lines: Vec<String>,
    theme: Theme,
) -> AnyElement<'static> {
    element! {
        View(flex_direction: FlexDirection::Column, width: 100pct) {
            View(flex_direction: FlexDirection::Row) {
                Text(content: file_path, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                #(suffix.map(|suffix| element! {
                    Text(content: suffix.to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
                }))
            }
            Divider(padding: 4u32)
            View(flex_direction: FlexDirection::Column) {
                #(lines.into_iter().map(|line| element! {
                    Text(content: line, color: theme.inactive, italic: true, wrap: TextWrap::NoWrap)
                }))
            }
        }
    }
    .into_any()
}

/// Maps to: CC `components/diff/DiffDetailView.tsx:25-140`
/// `DiffDetailView`.
#[component]
pub fn DiffDetailView(
    props: &DiffDetailViewProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let (columns, _) = hooks.use_terminal_size();
    let file_path_for_memo = props.file_path.clone();
    let syntax_context =
        hooks.use_memo(|| read_syntax_context(&props.file_path), file_path_for_memo);

    if props.is_untracked {
        return special_file_detail(
            props.file_path.clone(),
            Some(" (untracked)"),
            vec![
                "New file not yet staged.".to_string(),
                format!("Run `git add {}` to see line counts.", props.file_path),
            ],
            *theme,
        );
    }
    if props.is_binary {
        return special_file_detail(
            props.file_path.clone(),
            None,
            vec!["Binary file - cannot display diff".to_string()],
            *theme,
        );
    }
    if props.is_large_file {
        return special_file_detail(
            props.file_path.clone(),
            None,
            vec!["Large file - diff exceeds 1 MB limit".to_string()],
            *theme,
        );
    }

    let diff_width = (columns as usize).saturating_sub(4).max(1);
    let first_line = syntax_context.0.clone();
    let file_content = syntax_context.1.clone();
    let hunks = props.hunks.clone();

    element! {
        View(flex_direction: FlexDirection::Column, width: 100pct) {
            View(flex_direction: FlexDirection::Row) {
                Text(content: props.file_path.clone(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
                #(props.is_truncated.then(|| element! {
                    Text(content: " (truncated)".to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
                }))
            }
            Divider(padding: 4u32)
            View(flex_direction: FlexDirection::Column, width: 100pct) {
                #(if hunks.is_empty() {
                    vec![element! {
                        Text(content: "No diff content".to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
                    }.into_any()]
                } else {
                    hunks.into_iter().enumerate().map(|(index, patch)| element! {
                        View(key: index.to_string(), width: 100pct) {
                            StructuredDiff(
                                patch: patch,
                                file_path: props.file_path.clone(),
                                first_line: first_line.clone(),
                                file_content: file_content.clone(),
                                dim: false,
                                width: diff_width,
                            )
                        }
                    }.into_any()).collect::<Vec<_>>()
                })
            }
            #(props.is_truncated.then(|| element! {
                Text(
                    content: "… diff truncated (exceeded 400 line limit)".to_string(),
                    color: theme.inactive,
                    italic: true,
                    wrap: TextWrap::NoWrap,
                )
            }))
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render(props: DiffDetailViewProps) -> String {
        let current_theme = *theme::current();
        element! {
            ContextProvider(value: Context::owned(current_theme)) {
                // `use_app_state` is strict since P7 — mount the provider the
                // real tree has instead of relying on a silent default.
                crate::state::app_state::AppStateProvider(
                    children: crate::state::app_state::ProviderChildren::new(move || element! {
                        DiffDetailView(
                            file_path: props.file_path.clone(),
                            hunks: props.hunks.clone(),
                            is_large_file: props.is_large_file,
                            is_binary: props.is_binary,
                            is_truncated: props.is_truncated,
                            is_untracked: props.is_untracked,
                        )
                    }.into_any()),
                )
            }
        }
        .render(Some(80))
        .to_string()
    }

    #[test]
    fn diff_detail_view_special_file_copy_matches_official() {
        let untracked = render(DiffDetailViewProps {
            file_path: "new.rs".to_string(),
            is_untracked: true,
            ..DiffDetailViewProps::default()
        });
        assert!(untracked.contains("new.rs (untracked)"));
        assert!(untracked.contains("New file not yet staged."));
        assert!(untracked.contains("Run `git add new.rs` to see line counts."));

        let binary = render(DiffDetailViewProps {
            file_path: "logo.bin".to_string(),
            is_binary: true,
            ..DiffDetailViewProps::default()
        });
        assert!(binary.contains("Binary file - cannot display diff"));

        let large = render(DiffDetailViewProps {
            file_path: "large.txt".to_string(),
            is_large_file: true,
            ..DiffDetailViewProps::default()
        });
        assert!(large.contains("Large file - diff exceeds 1 MB limit"));
    }

    #[test]
    fn diff_detail_view_renders_structured_hunk_and_truncation_notice() {
        let text = render(DiffDetailViewProps {
            file_path: "src/lib.rs".to_string(),
            hunks: vec![StructuredDiffHunk {
                old_start: 1,
                old_lines: 1,
                new_start: 1,
                new_lines: 1,
                lines: vec!["-old".to_string(), "+new".to_string()],
            }],
            is_truncated: true,
            ..DiffDetailViewProps::default()
        });
        assert!(text.contains("src/lib.rs (truncated)"), "canvas=\n{text}");
        assert!(text.contains("-old"), "canvas=\n{text}");
        assert!(text.contains("+new"), "canvas=\n{text}");
        assert!(text.contains("… diff truncated (exceeded 400 line limit)"));
    }
}
