//! Maps to: CC `components/PromptInput/inputModes.ts:1-31`.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PromptInputMode {
    #[default]
    Prompt,
    Bash,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoryMode {
    #[default]
    Prompt,
    Bash,
}

pub fn prepend_mode_character_to_input(input: &str, mode: PromptInputMode) -> String {
    match mode {
        PromptInputMode::Bash => format!("!{input}"),
        PromptInputMode::Prompt => input.to_string(),
    }
}

pub fn get_mode_from_input(input: &str) -> HistoryMode {
    if input.starts_with('!') {
        HistoryMode::Bash
    } else {
        HistoryMode::Prompt
    }
}

pub fn get_value_from_input(input: &str) -> String {
    match get_mode_from_input(input) {
        HistoryMode::Prompt => input.to_string(),
        HistoryMode::Bash => input.get(1..).unwrap_or_default().to_string(),
    }
}

pub fn is_input_mode_character(input: &str) -> bool {
    input == "!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_mode_prefix_round_trips_exactly() {
        assert_eq!(
            prepend_mode_character_to_input("pwd", PromptInputMode::Bash),
            "!pwd"
        );
        assert_eq!(
            prepend_mode_character_to_input("pwd", PromptInputMode::Prompt),
            "pwd"
        );
        assert_eq!(get_mode_from_input("!pwd"), HistoryMode::Bash);
        assert_eq!(get_value_from_input("!pwd"), "pwd");
        assert_eq!(get_value_from_input("pwd"), "pwd");
        assert!(is_input_mode_character("!"));
        assert!(!is_input_mode_character("!!"));
    }
}
