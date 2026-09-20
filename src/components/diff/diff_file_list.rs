//! Maps to: CC `components/diff/DiffFileList.tsx:1-153`.

use crate::constants::figures;
use crate::hooks::use_diff_data::DiffFile;
use crate::utils::theme::Theme;
use crate::utils::truncate::truncate_start_to_width;
use iocraft::prelude::*;

const MAX_VISIBLE_FILES: usize = 5;

#[derive(Default, Props)]
pub struct DiffFileListProps {
    pub files: Vec<DiffFile>,
    pub selected_index: usize,
}

/// Maps to: CC `DiffFileList.tsx:20-36` scroll-window `useMemo`.
pub fn visible_file_window(file_count: usize, selected_index: usize) -> (usize, usize) {
    if file_count == 0 || file_count <= MAX_VISIBLE_FILES {
        return (0, file_count);
    }
    let mut start = selected_index.saturating_sub(MAX_VISIBLE_FILES / 2);
    let mut end = start + MAX_VISIBLE_FILES;
    if end > file_count {
        end = file_count;
        start = end.saturating_sub(MAX_VISIBLE_FILES);
    }
    (start, end)
}

fn plural_file(count: usize) -> &'static str {
    if count == 1 { "file" } else { "files" }
}

#[derive(Default, Props)]
struct FileItemProps {
    file: DiffFile,
    is_selected: bool,
    max_path_width: usize,
}

fn file_stats_element(file: &DiffFile, is_selected: bool, theme: Theme) -> AnyElement<'static> {
    let inactive = (!is_selected).then_some(theme.inactive);
    if file.is_untracked {
        return element! {
            Text(content: "untracked".to_string(), color: inactive, italic: true, wrap: TextWrap::NoWrap)
        }
        .into_any();
    }
    if file.is_binary {
        return element! {
            Text(content: "Binary file".to_string(), color: inactive, italic: true, wrap: TextWrap::NoWrap)
        }
        .into_any();
    }
    if file.is_large_file {
        return element! {
            Text(content: "Large file modified".to_string(), color: inactive, italic: true, wrap: TextWrap::NoWrap)
        }
        .into_any();
    }

    element! {
        View(flex_direction: FlexDirection::Row) {
            #(if file.lines_added > 0 {
                Some(element! {
                    Text(
                        content: format!("+{}", file.lines_added),
                        color: theme.diff_added_word,
                        weight: if is_selected { Weight::Bold } else { Weight::Normal },
                        wrap: TextWrap::NoWrap,
                    )
                })
            } else {
                None
            })
            #(if file.lines_added > 0 && file.lines_removed > 0 {
                Some(element! { Text(content: " ".to_string(), wrap: TextWrap::NoWrap) })
            } else {
                None
            })
            #(if file.lines_removed > 0 {
                Some(element! {
                    Text(
                        content: format!("-{}", file.lines_removed),
                        color: theme.diff_removed_word,
                        weight: if is_selected { Weight::Bold } else { Weight::Normal },
                        wrap: TextWrap::NoWrap,
                    )
                })
            } else {
                None
            })
            #(if file.is_truncated {
                Some(element! {
                    Text(content: " (truncated)".to_string(), color: inactive, wrap: TextWrap::NoWrap)
                })
            } else {
                None
            })
        }
    }
    .into_any()
}

/// Maps to: CC `DiffFileList.tsx:79-106` `FileItem` and
/// `DiffFileList.tsx:108-153` `FileStats`.
#[component]
fn FileItem(props: &FileItemProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let display_path = truncate_start_to_width(&props.file.path, props.max_path_width);
    let pointer = if props.is_selected {
        format!("{} ", figures::get().pointer)
    } else {
        "  ".to_string()
    };
    let line = format!("{pointer}{display_path}");
    let stats = file_stats_element(&props.file, props.is_selected, *theme);

    element! {
        View(flex_direction: FlexDirection::Row, width: 100pct) {
            Text(
                content: line,
                color: props.is_selected.then_some(theme.background),
                weight: if props.is_selected { Weight::Bold } else { Weight::Normal },
                invert: props.is_selected,
                wrap: TextWrap::NoWrap,
            )
            View(flex_grow: 1.0f32)
            #(vec![stats])
        }
    }
}

/// Maps to: CC `components/diff/DiffFileList.tsx:16-77`
/// `DiffFileList`.
#[component]
pub fn DiffFileList(props: &DiffFileListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let (columns, _) = hooks.use_terminal_size();
    let (start_index, end_index) = visible_file_window(props.files.len(), props.selected_index);

    if props.files.is_empty() {
        return element! {
            Text(content: "No changed files".to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
        }
        .into_any();
    }

    let needs_pagination = props.files.len() > MAX_VISIBLE_FILES;
    let has_more_above = start_index > 0;
    let has_more_below = end_index < props.files.len();
    let max_path_width = (columns as usize).saturating_sub(16 + 3 + 4).max(20);
    let files = props.files[start_index..end_index].to_vec();
    let more_above = if has_more_above {
        format!(" ↑ {start_index} more {}", plural_file(start_index))
    } else {
        " ".to_string()
    };
    let below_count = props.files.len() - end_index;
    let more_below = if has_more_below {
        format!(" ↓ {below_count} more {}", plural_file(below_count))
    } else {
        " ".to_string()
    };

    element! {
        View(flex_direction: FlexDirection::Column, width: 100pct) {
            #(needs_pagination.then(|| element! {
                Text(content: more_above, color: theme.inactive, wrap: TextWrap::NoWrap)
            }))
            #(files.into_iter().enumerate().map(|(index, file)| {
                let key = file.path.clone();
                element! {
                    View(key: key) {
                        FileItem(
                            file: file,
                            is_selected: start_index + index == props.selected_index,
                            max_path_width: max_path_width,
                        )
                    }
                }
            }))
            #(needs_pagination.then(|| element! {
                Text(content: more_below, color: theme.inactive, wrap: TextWrap::NoWrap)
            }))
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn file(path: &str, added: usize, removed: usize) -> DiffFile {
        DiffFile {
            path: path.to_string(),
            lines_added: added,
            lines_removed: removed,
            ..DiffFile::default()
        }
    }

    #[test]
    fn visible_file_window_matches_official_middle_and_end_clamping() {
        assert_eq!(visible_file_window(0, 0), (0, 0));
        assert_eq!(visible_file_window(5, 4), (0, 5));
        assert_eq!(visible_file_window(10, 5), (3, 8));
        assert_eq!(visible_file_window(10, 9), (5, 10));
    }

    #[test]
    fn diff_file_list_renders_official_pagination_selection_and_stats_copy() {
        let current_theme = *theme::current();
        let files = vec![
            file("one.rs", 1, 0),
            file("two.rs", 0, 2),
            file("three.rs", 3, 4),
            file("four.rs", 0, 0),
            file("five.rs", 0, 0),
            file("six.rs", 1, 1),
            file("seven.rs", 1, 0),
        ];
        let canvas = element! {
            ContextProvider(value: Context::owned(current_theme)) {
                DiffFileList(files: files, selected_index: 5usize)
            }
        }
        .render(Some(80));
        let text = canvas.to_string();

        assert!(text.contains("↑ 2 more files"), "canvas=\n{text}");
        assert!(!text.contains("↓ 1 more file"), "canvas=\n{text}");
        assert!(text.contains(&format!("{} six.rs", figures::get().pointer)));
        assert!(text.contains("+1 -1"));
    }

    #[test]
    fn diff_file_list_special_file_stats_match_official_copy() {
        let current_theme = *theme::current();
        let mut untracked = file("new.rs", 0, 0);
        untracked.is_untracked = true;
        let mut binary = file("logo.bin", 0, 0);
        binary.is_binary = true;
        let mut large = file("large.txt", 1, 0);
        large.is_large_file = true;
        let text = element! {
            ContextProvider(value: Context::owned(current_theme)) {
                DiffFileList(files: vec![untracked, binary, large], selected_index: 0usize)
            }
        }
        .render(Some(80))
        .to_string();

        assert!(text.contains("untracked"));
        assert!(text.contains("Binary file"));
        assert!(text.contains("Large file modified"));
    }
}
