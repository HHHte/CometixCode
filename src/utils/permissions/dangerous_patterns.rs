//! Maps to: CC `utils/permissions/dangerousPatterns.ts`.
//!
//! Shared dangerous shell prefix lists used by auto-mode permission setup.
//! The internal-only tail follows the official internal-distribution gate at
//! access time; callers should use `dangerous_bash_patterns()` rather
//! than assuming the base slice is exhaustive.

/// Maps to: CC `CROSS_PLATFORM_CODE_EXEC`.
pub const CROSS_PLATFORM_CODE_EXEC: &[&str] = &[
    "python", "python3", "python2", "node", "deno", "tsx", "ruby", "perl", "php", "lua", "npx",
    "bunx", "npm run", "yarn run", "pnpm run", "bun run", "bash", "sh", "ssh",
];

const DANGEROUS_BASH_BASE_TAIL: &[&str] = &["zsh", "fish", "eval", "exec", "env", "xargs", "sudo"];

const ANT_ONLY_DANGEROUS_BASH_PATTERNS: &[&str] = &[
    "fa run", "coo", "gh", "gh api", "curl", "wget", "git", "kubectl", "aws", "gcloud", "gsutil",
];

/// Pure audience form of CC `DANGEROUS_BASH_PATTERNS`.
pub fn dangerous_bash_patterns_for_audience(
    audience: crate::utils::build_profile::BuildAudience,
) -> Vec<&'static str> {
    let mut patterns = Vec::with_capacity(
        CROSS_PLATFORM_CODE_EXEC.len()
            + DANGEROUS_BASH_BASE_TAIL.len()
            + ANT_ONLY_DANGEROUS_BASH_PATTERNS.len(),
    );
    patterns.extend_from_slice(CROSS_PLATFORM_CODE_EXEC);
    patterns.extend_from_slice(DANGEROUS_BASH_BASE_TAIL);
    if crate::utils::build_profile::audience_has_internal_capability(
        audience,
        crate::utils::build_profile::InternalCapability::Permissions,
    ) {
        patterns.extend_from_slice(ANT_ONLY_DANGEROUS_BASH_PATTERNS);
    }
    patterns
}

/// Maps to: CC `DANGEROUS_BASH_PATTERNS` for this binary's profile.
pub fn dangerous_bash_patterns() -> Vec<&'static str> {
    dangerous_bash_patterns_for_audience(crate::utils::build_profile::build_audience())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dangerous_patterns_include_cross_platform_and_shell_entries() {
        let patterns = dangerous_bash_patterns_for_audience(
            crate::utils::build_profile::BuildAudience::External,
        );
        assert!(patterns.contains(&"python"));
        assert!(patterns.contains(&"npm run"));
        assert!(patterns.contains(&"sudo"));
        assert!(!patterns.contains(&"gh api"));
    }

    #[test]
    fn internal_build_adds_official_internal_only_patterns() {
        let patterns = dangerous_bash_patterns_for_audience(
            crate::utils::build_profile::BuildAudience::AnthropicInternal,
        );
        assert!(patterns.contains(&"fa run"));
        assert!(patterns.contains(&"gh api"));
        assert!(patterns.contains(&"gsutil"));
    }
}
