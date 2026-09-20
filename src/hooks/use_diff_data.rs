//! Maps to: CC `hooks/useDiffData.ts:10-110`.
//! Fetches current git statistics and hunks on mount, then derives the sorted
//! `DiffFile` list and official large/truncated/untracked flags.

use crate::types::message::StructuredDiffHunk;
use crate::utils::git_diff::{GitDiffResult, GitDiffStats, fetch_git_diff, fetch_git_diff_hunks};
use iocraft::prelude::*;
use std::collections::BTreeMap;

const MAX_LINES_PER_FILE: usize = 400;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffFile {
    pub path: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub is_binary: bool,
    pub is_large_file: bool,
    pub is_truncated: bool,
    pub is_new_file: bool,
    pub is_untracked: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffData {
    pub stats: Option<GitDiffStats>,
    pub files: Vec<DiffFile>,
    pub hunks: BTreeMap<String, Vec<StructuredDiffHunk>>,
    pub loading: bool,
}

/// Test/embedding equivalent of providing the already-resolved hook value.
/// Production `DiffDialog` does not install this context and always executes
/// the official on-mount git fetch.
#[derive(Clone, Debug)]
pub struct DiffDataOverride(pub DiffData);

/// Maps to: CC `hooks/useDiffData.ts:74-109` return-value `useMemo`.
pub fn derive_diff_data(
    diff_result: Option<&GitDiffResult>,
    hunks: &BTreeMap<String, Vec<StructuredDiffHunk>>,
    loading: bool,
) -> DiffData {
    let Some(diff_result) = diff_result else {
        return DiffData {
            loading,
            ..DiffData::default()
        };
    };

    let files = diff_result
        .per_file_stats
        .iter()
        .map(|(path, file_stats)| {
            let has_hunk_entry = hunks.contains_key(path);
            let is_large_file =
                !file_stats.is_binary && !file_stats.is_untracked && !has_hunk_entry;
            let total_lines = file_stats.added + file_stats.removed;
            let is_truncated =
                !is_large_file && !file_stats.is_binary && total_lines > MAX_LINES_PER_FILE;
            DiffFile {
                path: path.clone(),
                lines_added: file_stats.added,
                lines_removed: file_stats.removed,
                is_binary: file_stats.is_binary,
                is_large_file,
                is_truncated,
                is_new_file: false,
                is_untracked: file_stats.is_untracked,
            }
        })
        .collect();

    DiffData {
        stats: Some(diff_result.stats.clone()),
        files,
        hunks: hunks.clone(),
        loading: false,
    }
}

/// Maps to: CC `hooks/useDiffData.ts:34-110` `useDiffData`.
pub fn use_diff_data(hooks: &mut Hooks<'_, '_>) -> DiffData {
    let override_data = hooks
        .try_use_context::<DiffDataOverride>()
        .map(|value| value.0.clone());
    let diff_result = hooks.use_state(|| Option::<GitDiffResult>::None);
    let hunks = hooks.use_state(BTreeMap::<String, Vec<StructuredDiffHunk>>::new);
    let loading = hooks.use_state(|| true);
    let has_override = override_data.is_some();

    hooks.use_future({
        let mut diff_result = diff_result;
        let mut hunks = hunks;
        let mut loading = loading;
        async move {
            if has_override {
                return;
            }
            let (stats_result, hunks_result) =
                tokio::join!(fetch_git_diff(), fetch_git_diff_hunks());
            diff_result.set(stats_result);
            hunks.set(hunks_result);
            loading.set(false);
        }
    });

    if let Some(data) = override_data {
        return data;
    }
    let result = diff_result.read().clone();
    let hunk_snapshot = hunks.read().clone();
    derive_diff_data(result.as_ref(), &hunk_snapshot, loading.get())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::git_diff::PerFileStats;

    #[test]
    fn derive_diff_data_matches_official_large_truncated_and_untracked_flags() {
        let result = GitDiffResult {
            stats: GitDiffStats {
                files_count: 4,
                lines_added: 406,
                lines_removed: 2,
            },
            per_file_stats: BTreeMap::from([
                (
                    "a-large.rs".to_string(),
                    PerFileStats {
                        added: 2,
                        removed: 1,
                        ..PerFileStats::default()
                    },
                ),
                (
                    "b-truncated.rs".to_string(),
                    PerFileStats {
                        added: 401,
                        removed: 1,
                        ..PerFileStats::default()
                    },
                ),
                (
                    "c-binary.bin".to_string(),
                    PerFileStats {
                        is_binary: true,
                        ..PerFileStats::default()
                    },
                ),
                (
                    "d-new.rs".to_string(),
                    PerFileStats {
                        is_untracked: true,
                        ..PerFileStats::default()
                    },
                ),
            ]),
            hunks: BTreeMap::new(),
        };
        let hunks = BTreeMap::from([(
            "b-truncated.rs".to_string(),
            vec![StructuredDiffHunk::default()],
        )]);

        let data = derive_diff_data(Some(&result), &hunks, true);
        assert!(data.files[0].is_large_file);
        assert!(data.files[1].is_truncated);
        assert!(!data.files[2].is_large_file);
        assert!(data.files[3].is_untracked);
        assert!(!data.files[3].is_large_file);
        assert!(!data.loading);
    }

    #[test]
    fn derive_diff_data_preserves_official_loading_shape_without_result() {
        assert_eq!(
            derive_diff_data(None, &BTreeMap::new(), true),
            DiffData {
                loading: true,
                ..DiffData::default()
            }
        );
    }
}
