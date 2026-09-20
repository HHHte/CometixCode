//! User-Agent string helpers.
//! Maps to: CC `utils/userAgent.ts`.

/// Maps to: CC `utils/userAgent.ts#getClaudeCodeUserAgent`.
pub fn get_claude_code_user_agent() -> String {
    format!("claude-code/{}", crate::constants::product::VERSION)
}

#[cfg(test)]
mod tests {
    #[test]
    fn claude_code_user_agent_matches_official_prefix() {
        assert_eq!(
            super::get_claude_code_user_agent(),
            format!("claude-code/{}", crate::constants::product::VERSION)
        );
    }
}
