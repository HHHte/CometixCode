//! The workspace-trust hook gate.
//! Maps to: CC `utils/hooks.ts:286-300` `shouldSkipHookDueToTrust()`.
//!
//! All hooks require workspace trust in interactive mode.
//!
//! Ownership: this file is a port-side home, not a CC one — CC has no
//! `utils/hooks/security.ts` (`ls ../rebuild/src/utils/hooks/ | rg -i secur`
//! matches nothing), and its one resident is declared in `utils/hooks.ts`,
//! which this port maps to `services/hooks/mod.rs`. Booked as a relocation
//! rather than done here, because `mod.rs` is the shared hook entry.
//!
//! The two MANAGED-settings predicates that used to sit alongside it —
//! `should_disable_all_hooks_including_managed` and
//! `should_allow_managed_hooks_only` — are gone, and were never anything but a
//! second copy: CC declares both in `utils/hooks/hooksConfigSnapshot.ts`
//! (`:62`, `:83`), so `hooks_config_snapshot.rs` is the CC-named owner, and the
//! copies here had no production caller at all
//! (`load_hooks_config_from_settings_sources` and
//! `should_disable_all_hooks_including_managed_from_settings` both read the
//! snapshot module's). Two spellings of one admin control is how two policies
//! drift apart; these two still agreed when the copy was deleted (2026-08-30),
//! and the point is not to rely on that a second time. The two rows only the
//! deleted tests covered moved to the owner as
//! `managed_predicates_hold_with_no_policy_settings_at_all`.

/// Whether to skip hook execution due to missing workspace trust.
/// Maps to: CC `shouldSkipHookDueToTrust()` (hooks.ts:286-300).
///
/// In non-interactive (SDK) mode, trust is implicit → skip = false.
/// In interactive mode, skip if trust dialog has not been accepted.
pub fn should_skip_hook_due_to_trust(workspace_trusted: bool, is_non_interactive: bool) -> bool {
    if is_non_interactive {
        return false;
    }
    !workspace_trusted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_skips_in_interactive_untrusted() {
        assert!(should_skip_hook_due_to_trust(false, false));
    }

    #[test]
    fn trust_allows_in_interactive_trusted() {
        assert!(!should_skip_hook_due_to_trust(true, false));
    }

    #[test]
    fn trust_allows_in_non_interactive() {
        assert!(!should_skip_hook_due_to_trust(false, true));
    }
}
