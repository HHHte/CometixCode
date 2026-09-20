//! FileEdit tool input/output types.
//!
//! Maps to: CC `tools/FileEditTool/types.ts`.

use crate::types::message::StructuredDiffHunk;

/// Maps to: CC `FileEditInput` — parsed tool call input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEditInput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub replace_all: bool,
}

/// Maps to: CC `EditInput` — individual edit without `file_path`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditInput {
    pub old_string: String,
    pub new_string: String,
    pub replace_all: bool,
}

/// Runtime edit with `replace_all` always defined.
/// Maps to: CC `tools/FileEditTool/types.ts#FileEdit`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEdit {
    pub old_string: String,
    pub new_string: String,
    pub replace_all: bool,
}

impl From<EditInput> for FileEdit {
    fn from(value: EditInput) -> Self {
        Self {
            old_string: value.old_string,
            new_string: value.new_string,
            replace_all: value.replace_all,
        }
    }
}

impl From<&EditInput> for FileEdit {
    fn from(value: &EditInput) -> Self {
        Self {
            old_string: value.old_string.clone(),
            new_string: value.new_string.clone(),
            replace_all: value.replace_all,
        }
    }
}

/// Maps to: CC `FileEditOutput` / `outputSchema`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEditOutput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub original_file: String,
    pub structured_patch: Vec<StructuredDiffHunk>,
    pub user_modified: bool,
    pub replace_all: bool,
    pub git_diff: Option<crate::utils::git_diff::ToolUseDiff>,
    /// Rust transport fields for source-shaped context effects. They are not
    /// serialized into the official output schema.
    pub(crate) updated_file: String,
    pub(crate) read_timestamp_ms: i64,
    pub(crate) dynamic_skill_dirs: Vec<String>,
}

/// Rust transport for a post-discovery Edit failure. CC throws, but its
/// dynamic-skill trigger Set has already been mutated and must survive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditErrorOutput {
    pub(crate) content: String,
    pub(crate) dynamic_skill_dirs: Vec<String>,
}

pub type GitDiffSummary = crate::utils::git_diff::ToolUseDiff;
pub type GitDiffStatus = crate::utils::git_diff::ToolUseDiffStatus;

impl FileEditInput {
    /// Parse tool-call JSON into the official input shape.
    pub fn from_args(args: &serde_json::Value) -> Result<Self, String> {
        let file_path = args
            .get("file_path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Error editing file: missing file_path".to_string())?
            .to_string();
        let old_string = args
            .get("old_string")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Error editing file: missing old_string".to_string())?
            .to_string();
        let new_string = args
            .get("new_string")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Error editing file: missing new_string".to_string())?
            .to_string();
        let replace_all = args
            .get("replace_all")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        Ok(Self {
            file_path,
            old_string,
            new_string,
            replace_all,
        })
    }

    pub fn as_edit(&self) -> FileEdit {
        FileEdit {
            old_string: self.old_string.clone(),
            new_string: self.new_string.clone(),
            replace_all: self.replace_all,
        }
    }
}
