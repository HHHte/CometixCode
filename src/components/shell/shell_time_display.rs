//! Maps to: CC `components/shell/ShellTimeDisplay.tsx`.

use crate::utils::format::format_duration;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ShellTimeDisplayProps {
    pub elapsed_time_seconds: Option<u64>,
    pub timeout_ms: Option<u64>,
}

/// Maps to: CC `components/shell/ShellTimeDisplay.tsx#ShellTimeDisplay` text branches.
pub fn shell_time_display_text(
    elapsed_time_seconds: Option<u64>,
    timeout_ms: Option<u64>,
) -> Option<String> {
    if elapsed_time_seconds.is_none() && timeout_ms.is_none() {
        return None;
    }
    let timeout = timeout_ms.map(format_duration_hide_trailing_zeros);
    let Some(elapsed_seconds) = elapsed_time_seconds else {
        return Some(format!("(timeout {})", timeout.unwrap_or_default()));
    };
    let elapsed = format_duration(elapsed_seconds.saturating_mul(1000));
    match timeout {
        Some(timeout) => Some(format!("({elapsed} · timeout {timeout})")),
        None => Some(format!("({elapsed})")),
    }
}

fn format_duration_hide_trailing_zeros(ms: u64) -> String {
    let text = format_duration(ms);
    text.replace(" 0s", "")
        .replace(" 0m", "")
        .replace(" 0h", "")
}

#[component]
pub fn ShellTimeDisplay(props: &ShellTimeDisplayProps) -> impl Into<AnyElement<'static>> {
    match shell_time_display_text(props.elapsed_time_seconds, props.timeout_ms) {
        Some(text) => {
            element! { Text(content: text, dim: true, wrap: TextWrap::NoWrap) }.into_any()
        }
        None => element! { View(width: 0u32, height: 0u32) }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_time_display_text_matches_official_branches() {
        assert_eq!(shell_time_display_text(None, None), None);
        assert_eq!(
            shell_time_display_text(None, Some(60_000)),
            Some("(timeout 1m)".to_string())
        );
        assert_eq!(
            shell_time_display_text(Some(65), None),
            Some("(1m 5s)".to_string())
        );
        assert_eq!(
            shell_time_display_text(Some(65), Some(120_000)),
            Some("(1m 5s · timeout 2m)".to_string())
        );
    }
}
