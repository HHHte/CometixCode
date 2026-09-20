//! Maps to: CC `utils/suggestions/shellHistoryCompletion.ts`.
//!
//! Prompt history is already read by `utils/prompt_history`; this owner only
//! applies the source's Bash filtering, 60-second cache and exact-prefix rule.

use crate::utils::prompt_history::HistoryEntry;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(60);
const MAX_COMMANDS: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellHistoryMatch {
    pub full_command: String,
    pub suffix: String,
}

#[derive(Default)]
struct Cache {
    commands: Option<Vec<String>>,
    loaded_at: Option<Instant>,
}

static CACHE: LazyLock<Mutex<Cache>> = LazyLock::new(|| Mutex::new(Cache::default()));

fn shell_history_commands(entries: &[HistoryEntry]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    entries
        .iter()
        .filter_map(|entry| entry.display.strip_prefix('!'))
        .map(str::trim)
        .filter(|command| !command.is_empty())
        .filter(|command| seen.insert((*command).to_string()))
        .take(MAX_COMMANDS)
        .map(ToOwned::to_owned)
        .collect()
}

fn cached_commands() -> Vec<String> {
    if let Ok(cache) = CACHE.lock()
        && cache
            .loaded_at
            .is_some_and(|loaded_at| loaded_at.elapsed() < CACHE_TTL)
        && let Some(commands) = cache.commands.as_ref()
    {
        return commands.clone();
    }
    let commands = shell_history_commands(&crate::utils::prompt_history::get_history());
    if let Ok(mut cache) = CACHE.lock() {
        cache.commands = Some(commands.clone());
        cache.loaded_at = Some(Instant::now());
    }
    commands
}

/// Maps to CC `getShellHistoryCompletion`.
pub fn get_shell_history_completion(input: &str) -> Option<ShellHistoryMatch> {
    if input.len() < 2 || input.trim().is_empty() {
        return None;
    }
    cached_commands()
        .into_iter()
        .find(|command| command.starts_with(input) && command != input)
        .map(|full_command| ShellHistoryMatch {
            suffix: full_command[input.len()..].to_string(),
            full_command,
        })
}

/// Maps to CC `clearShellHistoryCache`.
pub fn clear_shell_history_cache() {
    if let Ok(mut cache) = CACHE.lock() {
        cache.commands = None;
        cache.loaded_at = None;
    }
}

/// Maps to CC `prependToShellHistoryCache`.
pub fn prepend_to_shell_history_cache(command: &str) {
    let Ok(mut cache) = CACHE.lock() else {
        return;
    };
    let Some(commands) = cache.commands.as_mut() else {
        return;
    };
    if let Some(index) = commands.iter().position(|entry| entry == command) {
        commands.remove(index);
    }
    commands.insert(0, command.to_string());
    commands.truncate(MAX_COMMANDS);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(display: &str) -> HistoryEntry {
        HistoryEntry {
            display: display.to_string(),
            timestamp_ms: 0,
            pasted_contents: Default::default(),
        }
    }

    #[test]
    fn shell_history_filter_dedupes_bash_entries_in_source_order() {
        let commands = shell_history_commands(&[
            entry("!git status"),
            entry("prompt"),
            entry("!git status"),
            entry("!git stash"),
        ]);
        assert_eq!(commands, vec!["git status", "git stash"]);
    }

    #[test]
    fn shell_history_match_requires_exact_prefix_and_two_char_input() {
        let entries = vec![entry("!ls -lah")];
        let commands = shell_history_commands(&entries);
        assert!(get_shell_history_completion("").is_none());
        assert!(get_shell_history_completion("l").is_none());
        assert_eq!(
            commands
                .into_iter()
                .find(|command| command.starts_with("ls "))
                .as_deref(),
            Some("ls -lah")
        );
    }
}
