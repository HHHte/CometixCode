//! Shell-prefix command formatting.
//!
//! Maps to: CC `utils/bash/shellPrefix.ts:1-27`.

/// Maps to CC `formatShellPrefixCommand(prefix, command)`.
pub fn format_shell_prefix_command(prefix: &str, command: &str) -> String {
    if let Some(space_before_dash) = prefix.rfind(" -").filter(|index| *index > 0) {
        let exec_path = &prefix[..space_before_dash];
        let args = &prefix[space_before_dash + 1..];
        format!(
            "{} {args} {}",
            crate::utils::bash::shell_quote::quote(&[exec_path]),
            crate::utils::bash::shell_quote::quote(&[command]),
        )
    } else {
        format!(
            "{} {}",
            crate::utils::bash::shell_quote::quote(&[prefix]),
            crate::utils::bash::shell_quote::quote(&[command]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_shell_prefix_matches_official_path_and_argument_split() {
        assert_eq!(
            format_shell_prefix_command("/usr/bin/bash -c", "echo hi"),
            "/usr/bin/bash -c 'echo hi'"
        );
        assert_eq!(
            format_shell_prefix_command("/path with spaces/bash", "echo hi"),
            "'/path with spaces/bash' 'echo hi'"
        );
    }
}
