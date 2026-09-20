//! Maps to: CC `tools/BashTool/destructiveCommandWarning.ts`.
//! Informational warning strings only; these do not affect permission logic.

use regex::Regex;
use std::sync::LazyLock;

struct DestructivePattern {
    pattern: LazyLock<Regex>,
    warning: &'static str,
}

macro_rules! destructive_pattern {
    ($regex:literal, $warning:literal) => {
        DestructivePattern {
            pattern: LazyLock::new(|| Regex::new($regex).expect("valid destructive regex")),
            warning: $warning,
        }
    };
}

static GIT_CLEAN_FORCE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bgit\s+clean\b[^;&|\n]*-[a-zA-Z]*f").expect("valid git clean regex")
});
static GIT_CLEAN_DRY_RUN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bgit\s+clean\b[^;&|\n]*(?:-[a-zA-Z]*n|--dry-run)")
        .expect("valid git clean dry-run regex")
});

static DESTRUCTIVE_PATTERNS: LazyLock<Vec<DestructivePattern>> = LazyLock::new(|| {
    vec![
        destructive_pattern!(
            r"\bgit\s+reset\s+--hard\b",
            "Note: may discard uncommitted changes"
        ),
        destructive_pattern!(
            r"\bgit\s+push\b[^;&|\n]*[ \t](--force|--force-with-lease|-f)\b",
            "Note: may overwrite remote history"
        ),
        destructive_pattern!(
            r"\bgit\s+checkout\s+(--\s+)?\.[ \t]*($|[;&|\n])",
            "Note: may discard all working tree changes"
        ),
        destructive_pattern!(
            r"\bgit\s+restore\s+(--\s+)?\.[ \t]*($|[;&|\n])",
            "Note: may discard all working tree changes"
        ),
        destructive_pattern!(
            r"\bgit\s+stash[ \t]+(drop|clear)\b",
            "Note: may permanently remove stashed changes"
        ),
        destructive_pattern!(
            r"\bgit\s+branch\s+(-D[ \t]|--delete\s+--force|--force\s+--delete)\b",
            "Note: may force-delete a branch"
        ),
        destructive_pattern!(
            r"\bgit\s+(commit|push|merge)\b[^;&|\n]*--no-verify\b",
            "Note: may skip safety hooks"
        ),
        destructive_pattern!(
            r"\bgit\s+commit\b[^;&|\n]*--amend\b",
            "Note: may rewrite the last commit"
        ),
        destructive_pattern!(
            r"(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*[rR][a-zA-Z]*f|(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*f[a-zA-Z]*[rR]",
            "Note: may recursively force-remove files"
        ),
        destructive_pattern!(
            r"(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*[rR]",
            "Note: may recursively remove files"
        ),
        destructive_pattern!(
            r"(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*f",
            "Note: may force-remove files"
        ),
        destructive_pattern!(
            r"(?i)\b(DROP|TRUNCATE)\s+(TABLE|DATABASE|SCHEMA)\b",
            "Note: may drop or truncate database objects"
        ),
        destructive_pattern!(
            r#"(?i)\bDELETE\s+FROM\s+\w+[ \t]*(;|"|'|\n|$)"#,
            "Note: may delete all rows from a database table"
        ),
        destructive_pattern!(
            r"\bkubectl\s+delete\b",
            "Note: may delete Kubernetes resources"
        ),
        destructive_pattern!(
            r"\bterraform\s+destroy\b",
            "Note: may destroy Terraform infrastructure"
        ),
    ]
});

/// Maps to CC `getDestructiveCommandWarning(command)`.
pub fn get_destructive_command_warning(command: &str) -> Option<&'static str> {
    if GIT_CLEAN_FORCE_RE.is_match(command) && !GIT_CLEAN_DRY_RUN_RE.is_match(command) {
        return Some("Note: may permanently delete untracked files");
    }
    DESTRUCTIVE_PATTERNS
        .iter()
        .find(|entry| entry.pattern.is_match(command))
        .map(|entry| entry.warning)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_command_warnings_match_official_examples() {
        assert_eq!(
            get_destructive_command_warning("git reset --hard HEAD~1"),
            Some("Note: may discard uncommitted changes")
        );
        assert_eq!(
            get_destructive_command_warning("git push origin main --force-with-lease"),
            Some("Note: may overwrite remote history")
        );
        assert_eq!(
            get_destructive_command_warning("rm -rf target"),
            Some("Note: may recursively force-remove files")
        );
        assert_eq!(
            get_destructive_command_warning("DELETE FROM users;"),
            Some("Note: may delete all rows from a database table")
        );
        assert!(get_destructive_command_warning("git status").is_none());
    }
}
