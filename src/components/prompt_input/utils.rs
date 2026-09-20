//! Maps to: CC `components/PromptInput/utils.ts:1-64`.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrintableKey {
    pub ctrl: bool,
    pub meta: bool,
    pub escape: bool,
    pub return_key: bool,
    pub tab: bool,
    pub backspace: bool,
    pub delete: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub page_up: bool,
    pub page_down: bool,
    pub home: bool,
    pub end: bool,
}

pub fn is_vim_mode_enabled() -> bool {
    crate::utils::config::load_global_config()
        .editor_mode
        .as_deref()
        == Some("vim")
}

pub fn get_newline_instructions() -> String {
    let env = crate::utils::env::get();
    if env.platform == crate::utils::env::Platform::MacOS
        && env.terminal.as_deref() == Some("Apple_Terminal")
    {
        return "shift + ⏎ for newline".to_string();
    }
    if crate::commands::terminal_setup::terminal_setup::is_shift_enter_key_binding_installed() {
        return "shift + ⏎ for newline".to_string();
    }
    if crate::commands::terminal_setup::terminal_setup::has_used_backslash_return() {
        "\\⏎ for newline".to_string()
    } else {
        "backslash (\\) + return (⏎) for newline".to_string()
    }
}

pub fn is_non_space_printable(input: &str, key: PrintableKey) -> bool {
    let control = key.ctrl
        || key.meta
        || key.escape
        || key.return_key
        || key.tab
        || key.backspace
        || key.delete
        || key.up
        || key.down
        || key.left
        || key.right
        || key.page_up
        || key.page_down
        || key.home
        || key.end;
    !control
        && !input.is_empty()
        && !input.chars().next().is_some_and(char::is_whitespace)
        && !input.starts_with('\u{1b}')
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn printable_gate_rejects_controls_whitespace_and_escape_sequences() {
        assert!(is_non_space_printable("a", PrintableKey::default()));
        assert!(!is_non_space_printable(" a", PrintableKey::default()));
        assert!(!is_non_space_printable("\u{1b}[A", PrintableKey::default()));
        assert!(!is_non_space_printable(
            "a",
            PrintableKey {
                ctrl: true,
                ..Default::default()
            }
        ));
    }
}
