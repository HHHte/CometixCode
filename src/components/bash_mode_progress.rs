//! Maps to: CC `components/BashModeProgress.tsx`.

use crate::components::message_response::MessageResponse;
use crate::components::messages::UserBashInputMessage;
use crate::components::shell::ShellProgressMessage;
use iocraft::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BashModeShellProgress {
    pub output: String,
    pub full_output: String,
    pub elapsed_time_seconds: Option<u64>,
    pub total_lines: Option<usize>,
    pub total_bytes: Option<u64>,
    pub timeout_ms: Option<u64>,
}

#[derive(Default, Props)]
pub struct BashModeProgressProps {
    pub input: String,
    pub progress: Option<BashModeShellProgress>,
    pub verbose: bool,
}

/// Maps to: CC `components/BashModeProgress.tsx#BashModeProgress`.
#[component]
pub fn BashModeProgress(props: &BashModeProgressProps) -> impl Into<AnyElement<'static>> {
    let progress = props.progress.clone();

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            UserBashInputMessage(command: props.input.clone(), add_margin: false)
            #(if let Some(progress) = progress {
                element! {
                    ShellProgressMessage(
                        full_output: progress.full_output,
                        output: progress.output,
                        elapsed_time_seconds: progress.elapsed_time_seconds,
                        total_lines: progress.total_lines,
                        total_bytes: progress.total_bytes,
                        timeout_ms: progress.timeout_ms,
                        verbose: props.verbose,
                    )
                }.into_any()
            } else {
                element! {
                    MessageResponse {
                        Text(content: "Running…".to_string(), dim: true, wrap: TextWrap::NoWrap)
                    }
                }.into_any()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render(props: BashModeProgressProps) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                BashModeProgress(
                    input: props.input,
                    progress: props.progress,
                    verbose: props.verbose,
                )
            }
        }
        .render(Some(100))
        .to_string()
    }

    #[test]
    fn bash_mode_progress_renders_input_and_running_fallback() {
        let text = render(BashModeProgressProps {
            input: "echo hi".to_string(),
            progress: None,
            verbose: false,
        });

        assert!(text.contains("! echo hi"), "canvas=\n{text}");
        assert!(text.contains("Running…"), "canvas=\n{text}");
    }

    #[test]
    fn bash_mode_progress_renders_shell_progress_snapshot() {
        let text = render(BashModeProgressProps {
            input: "printf".to_string(),
            progress: Some(BashModeShellProgress {
                output: "one\ntwo".to_string(),
                full_output: "one\ntwo".to_string(),
                elapsed_time_seconds: Some(2),
                total_lines: Some(2),
                total_bytes: Some(2048),
                timeout_ms: None,
            }),
            verbose: false,
        });

        assert!(text.contains("! printf"), "canvas=\n{text}");
        assert!(text.contains("one"), "canvas=\n{text}");
        assert!(text.contains("two"), "canvas=\n{text}");
        assert!(text.contains("(2s)"), "canvas=\n{text}");
        assert!(text.contains("2KB"), "canvas=\n{text}");
    }
}
