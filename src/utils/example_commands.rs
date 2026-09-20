//! Prompt onboarding examples.
//!
//! Maps to: CC `utils/exampleCommands.ts#getExampleCommandFromCache`.
//! This external build consumes already-cached project filenames only. It does
//! not run git history commands or mutate project configuration from the TUI.

const FALLBACK_FILE: &str = "<filepath>";

pub fn example_command_from_cache(example_files: Option<&[String]>, entropy: u64) -> String {
    let files = example_files.filter(|files| !files.is_empty());
    let file = files
        .and_then(|files| files.get((entropy as usize) % files.len()))
        .map(String::as_str)
        .unwrap_or(FALLBACK_FILE);
    let commands = [
        "fix lint errors".to_string(),
        "fix typecheck errors".to_string(),
        format!("how does {file} work?"),
        format!("refactor {file}"),
        "how do I log an error?".to_string(),
        format!("edit {file} to..."),
        format!("write a test for {file}"),
        "create a util logging.py that...".to_string(),
    ];
    format!(
        "Try \"{}\"",
        commands[((entropy ^ entropy.rotate_left(17)) as usize) % commands.len()]
    )
}

pub fn runtime_example_entropy() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_use_cached_files_or_official_placeholder_without_io() {
        let fallback = (0..32)
            .map(|entropy| example_command_from_cache(None, entropy))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(fallback.contains("<filepath>"));

        let cached = ["engine.rs".to_string()];
        let examples = (0..32)
            .map(|entropy| example_command_from_cache(Some(&cached), entropy))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(examples.contains("engine.rs"));
        assert!(!examples.contains("<filepath>"));
    }
}
