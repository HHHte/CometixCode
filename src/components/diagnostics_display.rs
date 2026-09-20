//! Diagnostics attachment display.
//!
//! Maps to: CC `components/DiagnosticsDisplay.tsx`.

use crate::components::ctrl_o_to_expand::ctrl_o_to_expand_hint;
use crate::components::message_response::MessageResponse;
use crate::services::lsp::types::DiagnosticFile;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Default, Props)]
pub struct DiagnosticsDisplayProps {
    pub files: Vec<DiagnosticFile>,
    pub verbose: bool,
}

#[component]
pub fn DiagnosticsDisplay(
    props: &DiagnosticsDisplayProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    if props.files.is_empty() {
        return element! { Fragment }.into_any();
    }

    if props.verbose {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        return element! {
            View(flex_direction: FlexDirection::Column) {
                #(props.files.iter().map(|file| {
                    let heading = diagnostics_display_file_heading(&file.uri, &cwd);
                    element! {
                        View(flex_direction: FlexDirection::Column) {
                            MessageResponse(content: heading, color: Some(theme.inactive))
                            #(file.diagnostics.iter().map(|diagnostic| {
                                element! {
                                    MessageResponse(
                                        content: format!("  {}", crate::services::diagnostic_tracking::format_single_diagnostic(diagnostic)),
                                        color: Some(theme.inactive),
                                    )
                                }
                            }))
                        }
                    }
                }))
            }
        }
        .into_any();
    }

    let summary = diagnostics_display_summary(&props.files);
    element! {
        MessageResponse(
            content: format!("{} {}", summary, ctrl_o_to_expand_hint()),
            color: Some(theme.inactive),
        )
    }
    .into_any()
}

/// Maps to: CC `DiagnosticsDisplay.tsx` normal-mode summary branch.
pub fn diagnostics_display_summary(files: &[DiagnosticFile]) -> String {
    crate::services::diagnostic_tracking::format_diagnostics_display_summary(files)
}

/// Maps to: CC `DiagnosticsDisplay.tsx` verbose file header formatting.
pub fn diagnostics_display_file_heading(uri: &str, cwd: &Path) -> String {
    let display_path = uri.replace("file://", "").replace("_claude_fs_right:", "");
    let relative_path = relative_path_for_display(&display_path, cwd);
    let uri_label = if uri.starts_with("file://") {
        "(file://)".to_string()
    } else if uri.starts_with("_claude_fs_right:") {
        "(claude_fs_right)".to_string()
    } else {
        format!("({})", uri.split(':').next().unwrap_or(uri))
    };
    format!("{relative_path} {uri_label}:")
}

fn relative_path_for_display(path: &str, cwd: &Path) -> String {
    let path = Path::new(path);
    if let Ok(stripped) = path.strip_prefix(cwd) {
        let rendered = stripped.display().to_string();
        if rendered.is_empty() {
            ".".to_string()
        } else {
            rendered
        }
    } else {
        path.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::lsp::types::{Diagnostic, DiagnosticPosition, DiagnosticRange};

    fn diagnostic(message: &str) -> Diagnostic {
        Diagnostic {
            message: message.to_string(),
            severity: "Error".to_string(),
            range: DiagnosticRange {
                start: DiagnosticPosition {
                    line: 4,
                    character: 2,
                },
                end: DiagnosticPosition {
                    line: 4,
                    character: 5,
                },
            },
            source: Some("rust".to_string()),
            code: Some("E0425".to_string()),
        }
    }

    #[test]
    fn diagnostics_display_summary_matches_official_pluralization() {
        let files = vec![
            DiagnosticFile {
                uri: "file:///repo/src/main.rs".to_string(),
                diagnostics: vec![diagnostic("a"), diagnostic("b")],
            },
            DiagnosticFile {
                uri: "file:///repo/src/lib.rs".to_string(),
                diagnostics: vec![diagnostic("c")],
            },
        ];

        assert_eq!(
            diagnostics_display_summary(&files),
            "Found 3 new diagnostic issues in 2 files"
        );
    }

    #[test]
    fn diagnostics_display_file_heading_matches_official_uri_labels() {
        let cwd = PathBuf::from("/repo");
        assert_eq!(
            diagnostics_display_file_heading("file:///repo/src/main.rs", &cwd),
            "src/main.rs (file://):"
        );
        assert_eq!(
            diagnostics_display_file_heading("_claude_fs_right:/repo/src/main.rs", &cwd),
            "src/main.rs (claude_fs_right):"
        );
        assert_eq!(
            diagnostics_display_file_heading("untitled:buffer", &cwd),
            "untitled:buffer (untitled):"
        );
    }
}
