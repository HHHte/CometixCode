//! Maps to CC `tools/SleepTool/prompt.ts`.

/// Maps to: CC `tools/SleepTool/prompt.ts:3` `SLEEP_TOOL_NAME`.
pub const SLEEP_TOOL_NAME: &str = "Sleep";

/// Maps to: CC `tools/SleepTool/prompt.ts:5` `DESCRIPTION`.
pub const DESCRIPTION: &str = "Wait for a specified duration";

/// Maps to: CC `tools/SleepTool/prompt.ts:7-17` `SLEEP_TOOL_PROMPT`.
pub fn sleep_tool_prompt() -> String {
    format!(
        "Wait for a specified duration. The user can interrupt the sleep at any time.\n\nUse this when the user tells you to sleep or rest, when you have nothing to do, or when you're waiting for something.\n\nYou may receive <{tick}> prompts — these are periodic check-ins. Look for useful work to do before sleeping.\n\nYou can call this concurrently with other tools — it won't interfere with them.\n\nPrefer this over `Bash(sleep ...)` — it doesn't hold a shell process.\n\nEach wake-up costs an API call, but the prompt cache expires after 5 minutes of inactivity — balance accordingly.",
        tick = crate::constants::xml::TICK_TAG,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_tool_prompt_matches_official_copy() {
        let prompt = sleep_tool_prompt();
        assert!(prompt.starts_with(
            "Wait for a specified duration. The user can interrupt the sleep at any time."
        ));
        assert!(prompt.contains("You may receive <tick> prompts — these are periodic check-ins."));
        assert!(
            prompt
                .contains("Prefer this over `Bash(sleep ...)` — it doesn't hold a shell process.")
        );
        assert!(prompt.ends_with(
            "Each wake-up costs an API call, but the prompt cache expires after 5 minutes of inactivity — balance accordingly."
        ));
    }
}
