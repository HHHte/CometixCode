//! Maps to: CC `components/permissions/FilePermissionDialog/ideDiffConfig.ts`.
//!
//! This is the typed boundary between file permission dialogs and IDE diff
//! editing. The current iocraft permission dialog does not yet open/edit IDE
//! tabs, but tool-specific permission requests can now build and apply the same
//! edit config objects that CC passes to `useDiffInIDE(...)`.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEdit {
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IDEDiffEditMode {
    Single,
    Multiple,
}

impl IDEDiffEditMode {
    pub fn official_value(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multiple => "multiple",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IDEDiffConfig {
    pub file_path: String,
    pub edits: Vec<FileEdit>,
    pub edit_mode: Option<IDEDiffEditMode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IDEDiffChangeInput {
    pub file_path: String,
    pub edits: Vec<FileEdit>,
}

/// Maps to: CC `ideDiffConfig.ts#IDEDiffSupport<TInput>`.
pub trait IDEDiffSupport<TInput> {
    fn get_config(&self, input: &TInput) -> IDEDiffConfig;
    fn apply_changes(&self, input: TInput, modified_edits: Vec<FileEdit>) -> TInput;
}

/// Maps to: CC `ideDiffConfig.ts#createSingleEditDiffConfig`.
pub fn create_single_edit_diff_config(
    file_path: impl Into<String>,
    old_string: impl Into<String>,
    new_string: impl Into<String>,
    replace_all: Option<bool>,
) -> IDEDiffConfig {
    IDEDiffConfig {
        file_path: file_path.into(),
        edits: vec![FileEdit {
            old_string: old_string.into(),
            new_string: new_string.into(),
            replace_all,
        }],
        edit_mode: Some(IDEDiffEditMode::Single),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_single_edit_diff_config_matches_official_shape() {
        let config = create_single_edit_diff_config("src/lib.rs", "old", "new", Some(true));
        assert_eq!(config.file_path, "src/lib.rs");
        assert_eq!(config.edits.len(), 1);
        assert_eq!(config.edits[0].old_string, "old");
        assert_eq!(config.edits[0].new_string, "new");
        assert_eq!(config.edits[0].replace_all, Some(true));
        assert_eq!(config.edit_mode, Some(IDEDiffEditMode::Single));
        assert_eq!(IDEDiffEditMode::Single.official_value(), "single");
    }
}
