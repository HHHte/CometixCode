//! Maps to: CC `tools/PowerShellTool/destructiveCommandWarning.ts`.
//!
//! Purely informational warning strings for the permission dialog — they do
//! not affect permission logic or auto-approval (source header, :1-5).
//!
//! The PowerShell-flavoured sibling of
//! `tools/bash_tool/destructive_command_warning.rs`. The tables are NOT the
//! same: this one anchors the removal cmdlets to statement start so `git rm
//! --force` cannot match (`destructiveCommandWarning.ts:14-20`), and adds the
//! PS-only `Clear-Content`, `Format-Volume`, `Clear-Disk`, `Stop-Computer`,
//! `Restart-Computer` and `Clear-RecycleBin` entries.
//!
//! Word boundaries are pinned to ASCII with `(?-u:\b)`: CC's literals carry no
//! `u` flag, so JS `\b` sees only `[A-Za-z0-9_]` as word characters, while the
//! `regex` crate's default `\b` is Unicode-aware.

use regex::Regex;
use std::sync::LazyLock;

enum Matcher {
    Pattern(Regex),
    /// CC uses one regex with a negative lookahead
    /// (`destructiveCommandWarning.ts:68-70`); the `regex` crate has no
    /// lookaround, so the assertion becomes a second pattern evaluated in the
    /// same list position.
    GitCleanForce {
        force: Regex,
        dry_run: Regex,
    },
}

impl Matcher {
    fn is_match(&self, command: &str) -> bool {
        match self {
            Self::Pattern(pattern) => pattern.is_match(command),
            Self::GitCleanForce { force, dry_run } => {
                force.is_match(command) && !dry_run.is_match(command)
            }
        }
    }
}

fn pattern(source: &str) -> Regex {
    Regex::new(source).expect("valid PowerShell destructive regex")
}

/// Maps to: CC `destructiveCommandWarning.ts:12-96` `DESTRUCTIVE_PATTERNS`.
/// Order is load-bearing — `getDestructiveCommandWarning` returns the first
/// match, so a command hitting two entries reports the earlier warning.
static DESTRUCTIVE_PATTERNS: LazyLock<Vec<(Matcher, &'static str)>> = LazyLock::new(|| {
    // Statement-start anchor + stopper shared by the four removal entries
    // (:21-40). The stopper adds `}` but NOT `)`: `}` ends a block so later
    // flags belong to a different statement, while `)` closes a path grouping
    // whose trailing flags are still this command's.
    const REMOVE_HEAD: &str =
        r"(?i)(?:^|[|;&\n({])\s*(Remove-Item|rm|del|rd|rmdir|ri)(?-u:\b)[^|;&\n}]*";
    vec![
        (
            Matcher::Pattern(pattern(&format!(
                "{REMOVE_HEAD}-Recurse(?-u:\\b)[^|;&\\n}}]*-Force(?-u:\\b)"
            ))),
            "Note: may recursively force-remove files",
        ),
        (
            Matcher::Pattern(pattern(&format!(
                "{REMOVE_HEAD}-Force(?-u:\\b)[^|;&\\n}}]*-Recurse(?-u:\\b)"
            ))),
            "Note: may recursively force-remove files",
        ),
        (
            Matcher::Pattern(pattern(&format!("{REMOVE_HEAD}-Recurse(?-u:\\b)"))),
            "Note: may recursively remove files",
        ),
        (
            Matcher::Pattern(pattern(&format!("{REMOVE_HEAD}-Force(?-u:\\b)"))),
            "Note: may force-remove files",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)Clear-Content(?-u:\b)[^|;&\n]*\*")),
            "Note: may clear content of multiple files",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)Format-Volume(?-u:\b)")),
            "Note: may format a disk volume",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)Clear-Disk(?-u:\b)")),
            "Note: may clear a disk",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)git\s+reset\s+--hard(?-u:\b)")),
            "Note: may discard uncommitted changes",
        ),
        (
            Matcher::Pattern(pattern(
                r"(?i)(?-u:\b)git\s+push(?-u:\b)[^|;&\n]*\s+(--force|--force-with-lease|-f)(?-u:\b)",
            )),
            "Note: may overwrite remote history",
        ),
        (
            Matcher::GitCleanForce {
                force: pattern(r"(?i)(?-u:\b)git\s+clean(?-u:\b)[^|;&\n]*-[a-zA-Z]*f"),
                dry_run: pattern(
                    r"(?i)(?-u:\b)git\s+clean(?-u:\b)[^|;&\n]*(?:-[a-zA-Z]*n|--dry-run)",
                ),
            },
            "Note: may permanently delete untracked files",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)git\s+stash\s+(drop|clear)(?-u:\b)")),
            "Note: may permanently remove stashed changes",
        ),
        (
            Matcher::Pattern(pattern(
                r"(?i)(?-u:\b)(DROP|TRUNCATE)\s+(TABLE|DATABASE|SCHEMA)(?-u:\b)",
            )),
            "Note: may drop or truncate database objects",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)Stop-Computer(?-u:\b)")),
            "Note: will shut down the computer",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)Restart-Computer(?-u:\b)")),
            "Note: will restart the computer",
        ),
        (
            Matcher::Pattern(pattern(r"(?i)(?-u:\b)Clear-RecycleBin(?-u:\b)")),
            "Note: permanently deletes recycled files",
        ),
    ]
});

/// Maps to: CC `destructiveCommandWarning.ts:102-109`
/// `getDestructiveCommandWarning(command)` — the first matching pattern's
/// warning, or `null` when nothing matches.
pub fn get_destructive_command_warning(command: &str) -> Option<&'static str> {
    DESTRUCTIVE_PATTERNS
        .iter()
        .find(|(matcher, _)| matcher.is_match(command))
        .map(|(_, warning)| *warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removal_warnings_match_official_flag_combinations() {
        assert_eq!(
            get_destructive_command_warning("Remove-Item -Recurse -Force .\\build"),
            Some("Note: may recursively force-remove files")
        );
        assert_eq!(
            get_destructive_command_warning("Remove-Item -Force -Recurse .\\build"),
            Some("Note: may recursively force-remove files")
        );
        assert_eq!(
            get_destructive_command_warning("rm -Recurse .\\build"),
            Some("Note: may recursively remove files")
        );
        assert_eq!(
            get_destructive_command_warning("del -Force .\\x"),
            Some("Note: may force-remove files")
        );
        // CC :20 — a `)` closing a path grouping must NOT stop the scan.
        assert_eq!(
            get_destructive_command_warning("Remove-Item (Join-Path $r \"tmp\") -Recurse -Force"),
            Some("Note: may recursively force-remove files")
        );
    }

    #[test]
    fn removal_patterns_are_anchored_to_statement_start() {
        // CC :14-20 — the anchor is why `git rm --force` does not warn; a bare
        // `\b` would match `rm` after any word boundary.
        assert_eq!(get_destructive_command_warning("git rm --force file"), None);
        // Scriptblock and group bodies still count as statement starts.
        assert_eq!(
            get_destructive_command_warning("if ($x) { rm -Force ./x }"),
            Some("Note: may force-remove files")
        );
        // `}` ends a block, so flags after it belong to another statement.
        assert_eq!(
            get_destructive_command_warning("if ($x) {rm} else {Write-Output -Force}"),
            None
        );
    }

    #[test]
    fn powershell_only_and_git_entries_match_official_warnings() {
        assert_eq!(
            get_destructive_command_warning("Clear-Content .\\logs\\*.log"),
            Some("Note: may clear content of multiple files")
        );
        // No wildcard → no warning (CC :44 requires the `*`).
        assert_eq!(
            get_destructive_command_warning("Clear-Content .\\a.log"),
            None
        );
        assert_eq!(
            get_destructive_command_warning("Format-Volume -DriveLetter D"),
            Some("Note: may format a disk volume")
        );
        assert_eq!(
            get_destructive_command_warning("Clear-Disk -Number 1"),
            Some("Note: may clear a disk")
        );
        assert_eq!(
            get_destructive_command_warning("git reset --hard HEAD~1"),
            Some("Note: may discard uncommitted changes")
        );
        assert_eq!(
            get_destructive_command_warning("git push origin main --force-with-lease"),
            Some("Note: may overwrite remote history")
        );
        assert_eq!(
            get_destructive_command_warning("git stash drop"),
            Some("Note: may permanently remove stashed changes")
        );
        assert_eq!(
            get_destructive_command_warning("DROP TABLE users"),
            Some("Note: may drop or truncate database objects")
        );
        assert_eq!(
            get_destructive_command_warning("Stop-Computer -Force"),
            Some("Note: will shut down the computer")
        );
        assert_eq!(
            get_destructive_command_warning("Restart-Computer"),
            Some("Note: will restart the computer")
        );
        assert_eq!(
            get_destructive_command_warning("Clear-RecycleBin -Confirm:$false"),
            Some("Note: permanently deletes recycled files")
        );
        assert_eq!(get_destructive_command_warning("Get-ChildItem ."), None);
    }

    #[test]
    fn git_clean_dry_run_suppresses_the_warning() {
        // CC :69 negative lookahead.
        assert_eq!(
            get_destructive_command_warning("git clean -fd"),
            Some("Note: may permanently delete untracked files")
        );
        assert_eq!(get_destructive_command_warning("git clean -nfd"), None);
        assert_eq!(
            get_destructive_command_warning("git clean --dry-run -f"),
            None
        );
    }

    #[test]
    fn first_match_wins_in_source_order() {
        // `git push --force` is CC entry 9, `git clean -f` entry 10, so the
        // push warning must win when a command carries both.
        assert_eq!(
            get_destructive_command_warning("git push --force; git clean -fd"),
            Some("Note: may overwrite remote history")
        );
    }

    #[test]
    fn word_boundaries_are_ascii_like_the_source_regexps() {
        // JS `\b` without the `u` flag only knows `[A-Za-z0-9_]`, so a
        // non-ASCII letter right after the cmdlet still forms a boundary.
        assert_eq!(
            get_destructive_command_warning("Clear-Diskä"),
            Some("Note: may clear a disk")
        );
        // Multi-byte input must not panic anywhere in the scan.
        assert_eq!(
            get_destructive_command_warning("Write-Output '日本語'"),
            None
        );
    }
}
