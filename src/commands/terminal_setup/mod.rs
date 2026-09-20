//! Maps to: CC `commands/terminalSetup/index.ts`.

pub mod terminal_setup;

/// Maps to: CC `commands/terminalSetup/index.ts` local
/// `NATIVE_CSIU_TERMINALS`; this command metadata table currently differs from
/// `terminalSetup.tsx` by not listing Warp.
const NATIVE_CSIU_TERMINALS: &[(&str, &str)] = &[
    ("ghostty", "Ghostty"),
    ("kitty", "Kitty"),
    ("iTerm.app", "iTerm2"),
    ("WezTerm", "WezTerm"),
];

/// Maps to: CC `commands/terminalSetup/index.ts` `description`.
pub fn terminal_setup_command_description_for(terminal: Option<&str>) -> &'static str {
    if terminal == Some("Apple_Terminal") {
        "Enable Option+Enter key binding for newlines and visual bell"
    } else {
        "Install Shift+Enter key binding for newlines"
    }
}

/// Maps to: CC `commands/terminalSetup/index.ts` `isHidden`.
pub fn terminal_setup_command_is_hidden_for(terminal: Option<&str>) -> bool {
    terminal.is_some()
        && NATIVE_CSIU_TERMINALS
            .iter()
            .any(|(key, _)| Some(*key) == terminal)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_setup_metadata_matches_official_index_boundary() {
        assert_eq!(
            terminal_setup_command_description_for(Some("Apple_Terminal")),
            "Enable Option+Enter key binding for newlines and visual bell"
        );
        assert_eq!(
            terminal_setup_command_description_for(Some("vscode")),
            "Install Shift+Enter key binding for newlines"
        );
        assert!(terminal_setup_command_is_hidden_for(Some("kitty")));
        assert!(terminal_setup_command_is_hidden_for(Some("WezTerm")));
        assert!(!terminal_setup_command_is_hidden_for(Some("WarpTerminal")));
        assert!(!terminal_setup_command_is_hidden_for(None));
    }
}
