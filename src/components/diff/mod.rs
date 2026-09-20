//! Maps to: CC `components/diff/`.
//! Official file/component boundaries are preserved in the three child
//! modules rather than folded into `StructuredDiff` or a command panel.

pub mod diff_detail_view;
pub mod diff_dialog;
pub mod diff_file_list;

pub use diff_detail_view::DiffDetailView;
pub use diff_dialog::{DiffDialog, DiffDialogDone};
pub use diff_file_list::DiffFileList;
