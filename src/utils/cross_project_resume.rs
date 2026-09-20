//! Maps to: CC `utils/crossProjectResume.ts`.

use crate::utils::bash::shell_quote::quote;
use crate::utils::session_storage::SessionSelection;
use std::path::Path;

/// Rust enum projection of CC `CrossProjectResumeResult`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrossProjectResumeResult {
    SameProject,
    SameRepoWorktree {
        project_path: String,
    },
    DifferentProject {
        project_path: String,
        command: String,
    },
}

/// Maps to: CC `utils/crossProjectResume.ts#checkCrossProjectResume`.
///
pub fn check_cross_project_resume_for_audience(
    selection: &SessionSelection,
    show_all_projects: bool,
    worktree_paths: &[String],
    audience: crate::utils::build_profile::BuildAudience,
) -> CrossProjectResumeResult {
    let current_project_path = crate::bootstrap::state::get_original_cwd()
        .display()
        .to_string();
    let Some(project_path) = selection
        .project_path
        .as_deref()
        .filter(|path| !path.is_empty())
    else {
        return CrossProjectResumeResult::SameProject;
    };

    if !show_all_projects || project_path == current_project_path {
        return CrossProjectResumeResult::SameProject;
    }

    // Maps to CC's staged rollout gate: external builds treat even same-repo
    // worktrees as cross-project and print a `cd ... && claude --resume ...`.
    let same_repo_worktree = crate::utils::build_profile::audience_has_internal_capability(
        audience,
        crate::utils::build_profile::InternalCapability::Ui,
    ) && worktree_paths
        .iter()
        .any(|worktree| path_is_or_under(project_path, worktree));
    if same_repo_worktree {
        return CrossProjectResumeResult::SameRepoWorktree {
            project_path: project_path.to_string(),
        };
    }

    CrossProjectResumeResult::DifferentProject {
        project_path: project_path.to_string(),
        command: format!(
            "cd {} && claude --resume {}",
            quote(&[project_path]),
            selection.session_id
        ),
    }
}

pub fn check_cross_project_resume(
    selection: &SessionSelection,
    show_all_projects: bool,
    worktree_paths: &[String],
) -> CrossProjectResumeResult {
    check_cross_project_resume_for_audience(
        selection,
        show_all_projects,
        worktree_paths,
        crate::utils::build_profile::build_audience(),
    )
}

fn path_is_or_under(path: &str, base: &str) -> bool {
    path == base || Path::new(path).starts_with(Path::new(base))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn check_cross_project_resume_matches_official_worktree_gate_and_command() {
        let previous_original_cwd = crate::bootstrap::state::get_original_cwd();
        crate::bootstrap::state::set_original_cwd("/repo");
        let worktree_selection = SessionSelection {
            session_id: "abc".to_string(),
            project_path: Some("/repo-wt/subdir".to_string()),
            file_path: PathBuf::from("/repo-wt/subdir/abc.jsonl"),
        };

        assert_eq!(
            check_cross_project_resume_for_audience(
                &worktree_selection,
                true,
                &["/repo".to_string(), "/repo-wt".to_string()],
                crate::utils::build_profile::BuildAudience::AnthropicInternal,
            ),
            CrossProjectResumeResult::SameRepoWorktree {
                project_path: "/repo-wt/subdir".to_string(),
            }
        );

        assert_eq!(
            check_cross_project_resume_for_audience(
                &worktree_selection,
                true,
                &["/repo".to_string(), "/repo-wt".to_string()],
                crate::utils::build_profile::BuildAudience::External,
            ),
            CrossProjectResumeResult::DifferentProject {
                project_path: "/repo-wt/subdir".to_string(),
                command: "cd /repo-wt/subdir && claude --resume abc".to_string(),
            }
        );

        let other_selection = SessionSelection {
            session_id: "abc".to_string(),
            project_path: Some("/other project".to_string()),
            file_path: PathBuf::from("/other project/abc.jsonl"),
        };
        assert_eq!(
            check_cross_project_resume(&other_selection, true, &["/repo".to_string()]),
            CrossProjectResumeResult::DifferentProject {
                project_path: "/other project".to_string(),
                command: "cd '/other project' && claude --resume abc".to_string(),
            }
        );

        crate::bootstrap::state::set_original_cwd(previous_original_cwd);
    }
}
