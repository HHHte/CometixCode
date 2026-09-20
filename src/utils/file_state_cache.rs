//! Canonical Read file-state cache.
//!
//! Maps to: CC `utils/fileStateCache.ts:1-142`.

use crate::utils::query_helpers::ReadFileStateEntry;
use std::path::{Component, Path, PathBuf};

/// Maps to: CC `utils/fileStateCache.ts:21` `READ_FILE_STATE_CACHE_SIZE`.
pub const READ_FILE_STATE_CACHE_SIZE: usize = 100;
/// Maps to: CC `utils/fileStateCache.ts:25` `DEFAULT_MAX_CACHE_SIZE_BYTES`.
pub const DEFAULT_MAX_CACHE_SIZE_BYTES: usize = 25 * 1024 * 1024;

/// Owned projection of CC `FileStateCache.dump()` plus its cache limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileStateCacheSnapshot {
    pub max_entries: usize,
    pub max_size_bytes: usize,
    /// Entries are retained from least-recently used to most-recently used.
    pub entries_lru_to_mru: Vec<ReadFileStateEntry>,
}

/// Maps to: CC `utils/fileStateCache.ts:33-100` `FileStateCache`.
///
/// The vector is the native Rust carrier for `lru-cache`: index zero is LRU,
/// the final entry is MRU. Key normalization, promotion, byte accounting, and
/// eviction live only in this owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileStateCache {
    entries: Vec<ReadFileStateEntry>,
    max_entries: usize,
    max_size_bytes: usize,
    calculated_size_bytes: usize,
}

impl Default for FileStateCache {
    fn default() -> Self {
        Self::with_size_limit(READ_FILE_STATE_CACHE_SIZE, DEFAULT_MAX_CACHE_SIZE_BYTES)
    }
}

impl FileStateCache {
    /// Maps to: CC `createFileStateCacheWithSizeLimit` (`fileStateCache.ts:108-114`).
    pub fn with_size_limit(max_entries: usize, max_size_bytes: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
            max_size_bytes,
            calculated_size_bytes: 0,
        }
    }

    /// Maps to: CC `cloneFileStateCache` + `FileStateCache.load`
    /// (`fileStateCache.ts:85-96,118-125`) through an owned Rust snapshot.
    pub fn from_snapshot(snapshot: FileStateCacheSnapshot) -> Self {
        let mut cache = Self::with_size_limit(snapshot.max_entries, snapshot.max_size_bytes);
        for entry in snapshot.entries_lru_to_mru {
            cache.set(Path::new(&entry.path.clone()), entry);
        }
        cache
    }

    /// Rust durable-Vec adapter for CC `FileStateCache.load`
    /// (`fileStateCache.ts:93-95`).
    pub fn from_entries(entries: Vec<ReadFileStateEntry>) -> Self {
        let mut cache = Self::default();
        cache.replace(entries);
        cache
    }

    /// Maps to: CC `FileStateCache.dump` (`fileStateCache.ts:89-91`), retaining
    /// the constructor limits needed by Rust detached-copy boundaries.
    pub fn snapshot(&self) -> FileStateCacheSnapshot {
        FileStateCacheSnapshot {
            max_entries: self.max_entries,
            max_size_bytes: self.max_size_bytes,
            entries_lru_to_mru: self.entries.clone(),
        }
    }

    /// Maps to: CC `FileStateCache.get` (`fileStateCache.ts:47-49`).
    pub fn get(&mut self, path: &Path) -> Option<ReadFileStateEntry> {
        let key = normalize_key(path);
        let index = self
            .entries
            .iter()
            .position(|entry| normalize_key(Path::new(&entry.path)) == key)?;
        let entry = self.entries.remove(index);
        let result = entry.clone();
        self.entries.push(entry);
        Some(result)
    }

    /// Maps to: CC `FileStateCache.set` (`fileStateCache.ts:51-54`).
    pub fn set(&mut self, path: &Path, mut entry: ReadFileStateEntry) {
        let key = normalize_key(path);
        entry.path = key.clone();
        let size = entry_size_bytes(&entry);
        if let Some(index) = self
            .entries
            .iter()
            .position(|existing| normalize_key(Path::new(&existing.path)) == key)
        {
            let removed = self.entries.remove(index);
            self.calculated_size_bytes = self
                .calculated_size_bytes
                .saturating_sub(entry_size_bytes(&removed));
        }
        // `lru-cache` deletes an existing key when its replacement exceeds
        // the configured maximum entry/cache size.
        if self.max_entries == 0 || self.max_size_bytes == 0 || size > self.max_size_bytes {
            return;
        }
        self.calculated_size_bytes = self.calculated_size_bytes.saturating_add(size);
        self.entries.push(entry);
        while self.entries.len() > self.max_entries
            || self.calculated_size_bytes > self.max_size_bytes
        {
            let removed = self.entries.remove(0);
            self.calculated_size_bytes = self
                .calculated_size_bytes
                .saturating_sub(entry_size_bytes(&removed));
        }
    }

    /// Rust entry-carrier adapter for CC `FileStateCache.set`
    /// (`fileStateCache.ts:44-47`).
    pub fn set_entry(&mut self, entry: ReadFileStateEntry) {
        let path = std::path::PathBuf::from(&entry.path);
        self.set(&path, entry);
    }

    /// Maps to: CC `FileStateCache.has` (`fileStateCache.ts:56-58`).
    pub fn has(&self, path: &Path) -> bool {
        let key = normalize_key(path);
        self.entries
            .iter()
            .any(|entry| normalize_key(Path::new(&entry.path)) == key)
    }

    /// Maps to: CC `FileStateCache.delete` (`fileStateCache.ts:60-62`).
    pub fn delete(&mut self, path: &Path) -> bool {
        let key = normalize_key(path);
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| normalize_key(Path::new(&entry.path)) == key)
        else {
            return false;
        };
        let removed = self.entries.remove(index);
        self.calculated_size_bytes = self
            .calculated_size_bytes
            .saturating_sub(entry_size_bytes(&removed));
        true
    }

    /// Maps to: CC `FileStateCache.clear` (`fileStateCache.ts:64-66`).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.calculated_size_bytes = 0;
    }

    /// Maps to: CC `FileStateCache.keys` (`fileStateCache.ts:81-83`).
    pub fn keys(&self) -> Vec<String> {
        self.entries
            .iter()
            .rev()
            .map(|entry| entry.path.clone())
            .collect()
    }

    /// Rust dump-order projection used by durable Vec adapters; maps to the
    /// LRU ordering retained by CC `FileStateCache.dump` (`:89-91`).
    pub fn entries_lru_to_mru(&self) -> Vec<ReadFileStateEntry> {
        self.entries.clone()
    }

    /// Maps to: CC `FileStateCache.load` (`fileStateCache.ts:93-95`) from the
    /// Rust durable-entry representation.
    pub fn replace(&mut self, entries: Vec<ReadFileStateEntry>) {
        self.clear();
        for entry in entries {
            self.set_entry(entry);
        }
    }

    /// Maps to: CC `FileStateCache.size` (`fileStateCache.ts:65-67`).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Rust predicate projection of CC `FileStateCache.size` (`:65-67`).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Maps to: CC `FileStateCache.max` (`fileStateCache.ts:69-71`).
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Maps to: CC `FileStateCache.maxSize` (`fileStateCache.ts:73-75`).
    pub fn max_size_bytes(&self) -> usize {
        self.max_size_bytes
    }

    /// Maps to: CC `FileStateCache.calculatedSize` (`fileStateCache.ts:77-79`).
    pub fn calculated_size_bytes(&self) -> usize {
        self.calculated_size_bytes
    }
}

/// Maps to: CC `normalize(key)` at each `FileStateCache` operation
/// (`fileStateCache.ts:40-55`).
fn normalize_key(path: &Path) -> String {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(output.components().next_back(), Some(Component::Normal(_))) {
                    output.pop();
                } else if !output.has_root() {
                    output.push("..");
                }
            }
            other => output.push(other.as_os_str()),
        }
    }
    let normalized = if output.as_os_str().is_empty() {
        ".".to_string()
    } else {
        output.display().to_string()
    };
    if cfg!(windows) {
        normalized.replace('/', "\\")
    } else {
        normalized
    }
}

fn entry_size_bytes(entry: &ReadFileStateEntry) -> usize {
    entry
        .content
        .as_ref()
        .map_or(1, |content| content.len().max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::query_helpers::{ReadFileStateEntry, ReadFileStateSource};

    fn entry(path: &str, content: &str) -> ReadFileStateEntry {
        ReadFileStateEntry {
            path: path.to_string(),
            content: Some(content.to_string()),
            timestamp_ms: Some(1),
            offset: Some(serde_json::json!(1)),
            limit: None,
            is_partial_view: false,
            source: ReadFileStateSource::Read,
        }
    }

    #[test]
    fn get_promotes_and_returns_current_value_like_official() {
        let mut cache = FileStateCache::with_size_limit(2, 1024);
        cache.set_entry(entry("/tmp/a/../a.txt", "a"));
        cache.set_entry(entry("/tmp/b.txt", "b"));

        assert_eq!(
            cache
                .get(Path::new("/tmp/a.txt"))
                .unwrap()
                .content
                .as_deref(),
            Some("a")
        );
        assert_eq!(cache.keys(), vec!["/tmp/a.txt", "/tmp/b.txt"]);

        cache.set_entry(entry("/tmp/c.txt", "c"));
        assert!(!cache.has(Path::new("/tmp/b.txt")));
        assert!(cache.has(Path::new("/tmp/a.txt")));
    }

    #[test]
    fn has_does_not_promote_but_keys_follow_mru_order_like_official() {
        let mut cache = FileStateCache::with_size_limit(2, 1024);
        cache.set_entry(entry("/tmp/a", "a"));
        cache.set_entry(entry("/tmp/b", "b"));
        assert!(cache.has(Path::new("/tmp/a")));
        assert_eq!(cache.keys(), vec!["/tmp/b", "/tmp/a"]);

        cache.set_entry(entry("/tmp/c", "c"));
        assert!(!cache.has(Path::new("/tmp/a")));
        assert_eq!(cache.keys(), vec!["/tmp/c", "/tmp/b"]);
    }

    #[test]
    fn set_accounts_utf8_bytes_and_evicts_once_like_official() {
        let mut cache = FileStateCache::with_size_limit(10, 4);
        cache.set_entry(entry("/tmp/a", "é"));
        cache.set_entry(entry("/tmp/b", "ab"));
        assert_eq!(cache.calculated_size_bytes(), 4);
        cache.set_entry(entry("/tmp/c", "x"));
        assert_eq!(cache.keys(), vec!["/tmp/c", "/tmp/b"]);
        assert_eq!(cache.calculated_size_bytes(), 3);
    }

    #[test]
    fn normalization_preserves_leading_parent_segments_and_path_case() {
        let mut cache = FileStateCache::with_size_limit(10, 1024);
        cache.set_entry(entry("a/../../Mixed.txt", "value"));
        assert!(cache.has(Path::new("../Mixed.txt")));
        assert!(!cache.has(Path::new("../mixed.txt")));
    }

    #[test]
    fn oversized_replacement_removes_the_previous_value() {
        let mut cache = FileStateCache::with_size_limit(10, 4);
        cache.set_entry(entry("/tmp/a", "ok"));
        cache.set_entry(entry("/tmp/a", "oversized"));
        assert!(!cache.has(Path::new("/tmp/a")));
        assert_eq!(cache.calculated_size_bytes(), 0);
    }
}
