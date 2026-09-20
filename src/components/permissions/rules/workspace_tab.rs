//! Maps to: CC `components/permissions/rules/WorkspaceTab.tsx`.
//! Additional directories are projected from `ToolPermissionContext`, while
//! explicit snapshots remain available for isolated component callers/tests.

use crate::components::custom_select::{
    Select, SelectInputOptionMeta, SelectLayout, SelectOptionData, UseSelectInputOptions,
    UseSelectStateProps, use_select_input, use_select_state,
};
use crate::tool::ToolPermissionContext;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryItem {
    pub path: String,
    pub is_current: bool,
    pub is_deletable: bool,
}

pub const ADD_DIRECTORY_VALUE: &str = "add-directory";

/// Maps to: CC `WorkspaceTab` `options` memo.
pub fn workspace_tab_options(additional_directories: &[DirectoryItem]) -> Vec<SelectOptionData> {
    let mut options = additional_directories
        .iter()
        .map(|directory| SelectOptionData {
            label: directory.path.clone(),
            value: directory.path.clone(),
            description: None,
            dim_description: true,
            disabled: !directory.is_deletable,
            input: None,
        })
        .collect::<Vec<_>>();
    options.push(SelectOptionData {
        label: "Add directory…".to_string(),
        value: ADD_DIRECTORY_VALUE.to_string(),
        description: None,
        dim_description: true,
        disabled: false,
        input: None,
    });
    options
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceTabSelection {
    AddDirectory,
    RemoveDirectory(String),
    Ignore,
}

/// Maps to: CC `WorkspaceTab.tsx#handleDirectorySelect`.
pub fn workspace_tab_selection(
    selected_value: &str,
    additional_directories: &[DirectoryItem],
) -> WorkspaceTabSelection {
    if selected_value == ADD_DIRECTORY_VALUE {
        return WorkspaceTabSelection::AddDirectory;
    }
    additional_directories
        .iter()
        .find(|directory| directory.path == selected_value && directory.is_deletable)
        .map(|directory| WorkspaceTabSelection::RemoveDirectory(directory.path.clone()))
        .unwrap_or(WorkspaceTabSelection::Ignore)
}

#[derive(Default, Props)]
pub struct WorkspaceTabProps<'a> {
    pub original_cwd: String,
    pub tool_permission_context: Option<ToolPermissionContext>,
    pub additional_directories: Vec<DirectoryItem>,
    /// Maps to: CC `headerFocused` from `useTabHeaderFocus()` — Rust has no
    /// Tabs context yet, so the parent passes the flag down.
    pub header_focused: bool,
    /// Maps to: CC `onRequestAddDirectory` (WorkspaceTab.tsx:52-55).
    pub on_request_add_directory: HandlerMut<'a, ()>,
    /// Maps to: CC `onRequestRemoveDirectory` (WorkspaceTab.tsx:57-62).
    pub on_request_remove_directory: HandlerMut<'a, String>,
    /// Maps to: CC `handleCancel` → `onExit('Workspace dialog dismissed')`
    /// (WorkspaceTab.tsx:67-70); the exit result stays with the caller.
    pub on_cancel: HandlerMut<'a, ()>,
    /// Maps to: CC `focusHeader` from `useTabHeaderFocus()`, wired to the
    /// Select's `onUpFromFirstItem` (WorkspaceTab.tsx:100).
    pub on_focus_header: HandlerMut<'a, ()>,
}

/// Maps to: CC `WorkspaceTab` render path (WorkspaceTab.tsx:28-105).
#[component]
pub fn WorkspaceTab<'a>(
    props: &mut WorkspaceTabProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let additional_directories = if props.additional_directories.is_empty() {
        props
            .tool_permission_context
            .as_ref()
            .map(|context| {
                context
                    .additional_working_directories
                    .keys()
                    .map(|path| DirectoryItem {
                        path: path.clone(),
                        is_current: false,
                        is_deletable: true,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        props.additional_directories.clone()
    };
    let original_cwd = if props.original_cwd.is_empty() {
        crate::bootstrap::state::get_original_cwd()
            .to_string_lossy()
            .into_owned()
    } else {
        props.original_cwd.clone()
    };
    let options = workspace_tab_options(&additional_directories);
    // Maps to: CC WorkspaceTab.tsx:95-102 — autonomous `<Select options
    // onChange={handleDirectorySelect} onCancel={handleCancel}
    // visibleOptionCount={Math.min(10, options.length)}
    // onUpFromFirstItem={focusHeader} isDisabled={headerFocused}/>`.
    let state = use_select_state(
        &mut hooks,
        UseSelectStateProps {
            visible_option_count: Some(10.min(options.len())),
            values: options.iter().map(|option| option.value.clone()).collect(),
            default_value: None,
            focus_value: None,
        },
    );
    let events = use_select_input(
        &mut hooks,
        state,
        UseSelectInputOptions {
            is_disabled: props.header_focused,
            has_on_cancel: true,
            has_on_up_from_first_item: true,
            option_metas: options
                .iter()
                .map(|option| SelectInputOptionMeta {
                    value: option.value.clone(),
                    disabled: option.disabled,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        },
    );

    // Maps to: CC WorkspaceTab.tsx:50-65 `handleDirectorySelect` —
    // 'add-directory' requests the add flow, a deletable directory requests
    // removal, anything else is ignored.
    if let Some(value) = events.take_accepted() {
        match workspace_tab_selection(&value, &additional_directories) {
            WorkspaceTabSelection::AddDirectory => (props.on_request_add_directory)(()),
            WorkspaceTabSelection::RemoveDirectory(path) => {
                (props.on_request_remove_directory)(path)
            }
            WorkspaceTabSelection::Ignore => {}
        }
    }
    // Maps to: CC WorkspaceTab.tsx:98 `onCancel={handleCancel}`.
    if events.take_cancelled() {
        (props.on_cancel)(());
    }
    // Maps to: CC WorkspaceTab.tsx:100 `onUpFromFirstItem={focusHeader}`.
    if events.take_up_from_first_item() {
        (props.on_focus_header)(());
    }

    let navigation = state.navigation.snapshot();
    let focused_index = navigation.focused_index().unwrap_or(0);
    element! {
        View(flex_direction: FlexDirection::Column, margin_bottom: 1u32) {
            View(flex_direction: FlexDirection::Row, margin_top: 1u32, margin_left: 2u32) {
                Text(content: format!("-  {original_cwd}"))
                Text(content: "(Original working directory)".to_string(), color: theme.inactive)
            }
            Select(
                options: options.clone(),
                focused_index: focused_index,
                visible_option_count: navigation.visible_option_count,
                visible_from_index: navigation.visible_from_index,
                is_disabled: props.header_focused,
                layout: SelectLayout::Compact,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn dirs() -> Vec<DirectoryItem> {
        vec![DirectoryItem {
            path: "/repo/extra".to_string(),
            is_current: false,
            is_deletable: true,
        }]
    }

    #[test]
    fn workspace_tab_options_and_selection_match_official() {
        let options = workspace_tab_options(&dirs());
        assert_eq!(options[0].label, "/repo/extra");
        assert_eq!(options[1].value, ADD_DIRECTORY_VALUE);
        assert_eq!(
            workspace_tab_selection("/repo/extra", &dirs()),
            WorkspaceTabSelection::RemoveDirectory("/repo/extra".to_string())
        );
        assert_eq!(
            workspace_tab_selection(ADD_DIRECTORY_VALUE, &dirs()),
            WorkspaceTabSelection::AddDirectory
        );
    }

    #[test]
    fn workspace_tab_renders_original_cwd_and_add_directory() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                WorkspaceTab(original_cwd: "/repo".to_string(), additional_directories: dirs())
            }
        }
        .render(Some(100))
        .to_string();
        assert!(text.contains("/repo"), "canvas=\n{text}");
        assert!(
            text.contains("Original working directory"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("❯ 1. /repo/extra"),
            "shared Select should render the focused pointer and index; canvas=\n{text}"
        );
        assert!(text.contains("2. Add directory…"), "canvas=\n{text}");
    }
}
