//! Maps to: CC `tools/BashTool/toolName.ts`.
//! Kept separate to mirror the official circular-dependency breaker.

pub const BASH_TOOL_NAME: &str = "Bash";

#[cfg(test)]
mod tests {
    #[test]
    fn bash_tool_name_matches_official() {
        assert_eq!(super::BASH_TOOL_NAME, "Bash");
    }
}
