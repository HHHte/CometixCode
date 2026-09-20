//! Maps to: CC `components/agents/ToolSelector.tsx:1-478`.

use crate::components::design_system::divider::Divider;
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use crate::services::mcp::mcp_string_utils::mcp_info_from_string;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentToolOption {
    pub name: String,
    /// Projection of `filterToolsForAgent(...)`.
    pub available_to_custom_agent: bool,
}

impl AgentToolOption {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            available_to_custom_agent: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolBucket {
    ReadOnly,
    Edit,
    Execution,
    Mcp,
    Other,
}

fn bucket_for(name: &str) -> Option<ToolBucket> {
    const READ_ONLY: &[&str] = &[
        "Glob",
        "Grep",
        "ExitPlanMode",
        "Read",
        "WebFetch",
        "TodoWrite",
        "WebSearch",
        "TaskStop",
        "TaskOutput",
        "ListMcpResources",
        "ReadMcpResource",
    ];
    const EDIT: &[&str] = &["Edit", "Write", "NotebookEdit"];
    if mcp_info_from_string(name).is_some() {
        Some(ToolBucket::Mcp)
    } else if READ_ONLY.contains(&name) {
        Some(ToolBucket::ReadOnly)
    } else if EDIT.contains(&name) {
        Some(ToolBucket::Edit)
    } else if name == crate::tools::bash_tool::tool_name::BASH_TOOL_NAME {
        Some(ToolBucket::Execution)
    } else if name == "Agent" || name == "Task" {
        None
    } else {
        Some(ToolBucket::Other)
    }
}

fn bucket_name(bucket: ToolBucket) -> &'static str {
    match bucket {
        ToolBucket::ReadOnly => "Read-only tools",
        ToolBucket::Edit => "Edit tools",
        ToolBucket::Execution => "Execution tools",
        ToolBucket::Mcp => "MCP tools",
        ToolBucket::Other => "Other tools",
    }
}

#[derive(Clone, Debug)]
enum ItemAction {
    Continue,
    ToggleNames(Vec<String>, bool),
    ToggleAdvanced,
    ToggleTool(String),
    Header,
}

#[derive(Clone, Debug)]
struct NavigableItem {
    id: String,
    label: String,
    action: ItemAction,
    is_toggle: bool,
    is_header: bool,
}

fn selected_count(names: &[String], selected: &HashSet<String>) -> usize {
    names.iter().filter(|name| selected.contains(*name)).count()
}

fn build_items(
    tools: &[AgentToolOption],
    valid_selected: &[String],
    show_individual: bool,
) -> Vec<NavigableItem> {
    let selected = valid_selected.iter().cloned().collect::<HashSet<_>>();
    let all_names = tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let all_selected =
        !all_names.is_empty() && selected_count(&all_names, &selected) == all_names.len();
    let checkbox = crate::constants::figures::get();
    let mut items = vec![
        NavigableItem {
            id: "continue".to_string(),
            label: "Continue".to_string(),
            action: ItemAction::Continue,
            is_toggle: false,
            is_header: false,
        },
        NavigableItem {
            id: "bucket-all".to_string(),
            label: format!(
                "{} All tools",
                if all_selected {
                    checkbox.checkbox_on
                } else {
                    checkbox.checkbox_off
                }
            ),
            action: ItemAction::ToggleNames(all_names.clone(), !all_selected),
            is_toggle: false,
            is_header: false,
        },
    ];
    let order = [
        ToolBucket::ReadOnly,
        ToolBucket::Edit,
        ToolBucket::Execution,
        ToolBucket::Mcp,
        ToolBucket::Other,
    ];
    for bucket in order {
        let names = tools
            .iter()
            .filter(|tool| bucket_for(&tool.name) == Some(bucket))
            .map(|tool| tool.name.clone())
            .collect::<Vec<_>>();
        if names.is_empty() {
            continue;
        }
        let full = selected_count(&names, &selected) == names.len();
        items.push(NavigableItem {
            id: format!("bucket-{bucket:?}"),
            label: format!(
                "{} {}",
                if full {
                    checkbox.checkbox_on
                } else {
                    checkbox.checkbox_off
                },
                bucket_name(bucket)
            ),
            action: ItemAction::ToggleNames(names, !full),
            is_toggle: false,
            is_header: false,
        });
    }
    items.push(NavigableItem {
        id: "toggle-individual".to_string(),
        label: if show_individual {
            "Hide advanced options"
        } else {
            "Show advanced options"
        }
        .to_string(),
        action: ItemAction::ToggleAdvanced,
        is_toggle: true,
        is_header: false,
    });
    if !show_individual {
        return items;
    }

    let mut mcp_servers = BTreeMap::<String, Vec<String>>::new();
    for tool in tools {
        if let Some(info) = mcp_info_from_string(&tool.name) {
            mcp_servers
                .entry(info.server_name)
                .or_default()
                .push(tool.name.clone());
        }
    }
    if !mcp_servers.is_empty() {
        items.push(NavigableItem {
            id: "mcp-servers-header".to_string(),
            label: "MCP Servers:".to_string(),
            action: ItemAction::Header,
            is_toggle: false,
            is_header: true,
        });
        for (server, names) in mcp_servers {
            let full = selected_count(&names, &selected) == names.len();
            items.push(NavigableItem {
                id: format!("mcp-server-{server}"),
                label: format!(
                    "{} {server} ({} {})",
                    if full {
                        checkbox.checkbox_on
                    } else {
                        checkbox.checkbox_off
                    },
                    names.len(),
                    if names.len() == 1 { "tool" } else { "tools" }
                ),
                action: ItemAction::ToggleNames(names, !full),
                is_toggle: false,
                is_header: false,
            });
        }
        items.push(NavigableItem {
            id: "tools-header".to_string(),
            label: "Individual Tools:".to_string(),
            action: ItemAction::Header,
            is_toggle: false,
            is_header: true,
        });
    }
    for tool in tools {
        let display = mcp_info_from_string(&tool.name)
            .map(|info| {
                format!(
                    "{} ({})",
                    info.tool_name.unwrap_or_else(|| tool.name.clone()),
                    info.server_name
                )
            })
            .unwrap_or_else(|| tool.name.clone());
        items.push(NavigableItem {
            id: format!("tool-{}", tool.name),
            label: format!(
                "{} {display}",
                if selected.contains(&tool.name) {
                    checkbox.checkbox_on
                } else {
                    checkbox.checkbox_off
                }
            ),
            action: ItemAction::ToggleTool(tool.name.clone()),
            is_toggle: false,
            is_header: false,
        });
    }
    items
}

#[derive(Clone, Copy, Debug)]
enum InputAction {
    Previous,
    Next,
    Accept,
    Cancel,
}

#[derive(Default, Props)]
pub struct ToolSelectorProps<'a> {
    pub tools: Vec<AgentToolOption>,
    pub initial_tools: Option<Vec<String>>,
    pub on_complete: HandlerMut<'a, Option<Vec<String>>>,
    pub on_cancel: HandlerMut<'a, ()>,
}

#[component]
pub fn ToolSelector<'a>(
    props: &mut ToolSelectorProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let custom_tools = props
        .tools
        .iter()
        .filter(|tool| tool.available_to_custom_agent && bucket_for(&tool.name).is_some())
        .cloned()
        .collect::<Vec<_>>();
    let all_names = custom_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let initial = props
        .initial_tools
        .clone()
        .filter(|tools| !tools.iter().any(|tool| tool == "*"))
        .unwrap_or_else(|| all_names.clone());
    let mut selected_tools = hooks.use_state(move || initial);
    let mut focus_index = hooks.use_state(|| 0usize);
    let mut show_individual = hooks.use_state(|| false);
    let mut pending_input = hooks.use_state(|| None::<InputAction>);
    let mut pending_complete = hooks.use_state(|| None::<Option<Vec<String>>>);
    let mut pending_cancel = hooks.use_state(|| false);

    let valid_selected = selected_tools
        .read()
        .iter()
        .filter(|name| all_names.contains(name))
        .cloned()
        .collect::<Vec<_>>();
    let items = build_items(&custom_tools, &valid_selected, show_individual.get());
    if focus_index.get() >= items.len() {
        focus_index.set(items.len().saturating_sub(1));
    }

    let completion = {
        let value = pending_complete.read();
        value.clone()
    };
    if let Some(value) = completion {
        pending_complete.set(None);
        (props.on_complete)(value);
    }
    if pending_cancel.get() {
        pending_cancel.set(false);
        if props.on_cancel.is_default() {
            (props.on_complete)(props.initial_tools.clone());
        } else {
            (props.on_cancel)(());
        }
    }
    let input_action = {
        let value = pending_input.read();
        *value
    };
    if let Some(action) = input_action {
        pending_input.set(None);
        match action {
            InputAction::Previous => {
                let mut next = focus_index.get().saturating_sub(1);
                while next > 0 && items.get(next).is_some_and(|item| item.is_header) {
                    next -= 1;
                }
                focus_index.set(next);
            }
            InputAction::Next => {
                let mut next = (focus_index.get() + 1).min(items.len().saturating_sub(1));
                while next < items.len().saturating_sub(1)
                    && items.get(next).is_some_and(|item| item.is_header)
                {
                    next += 1;
                }
                focus_index.set(next);
            }
            InputAction::Cancel => pending_cancel.set(true),
            InputAction::Accept => {
                if let Some(item) = items.get(focus_index.get()) {
                    match &item.action {
                        ItemAction::Continue => {
                            let all_selected = valid_selected.len() == all_names.len()
                                && all_names.iter().all(|name| valid_selected.contains(name));
                            pending_complete.set(Some(if all_selected {
                                None
                            } else {
                                Some(valid_selected.clone())
                            }));
                        }
                        ItemAction::ToggleNames(names, select) => {
                            let mut next = selected_tools.read().clone();
                            if *select {
                                for name in names {
                                    if !next.contains(name) {
                                        next.push(name.clone());
                                    }
                                }
                            } else {
                                next.retain(|name| !names.contains(name));
                            }
                            selected_tools.set(next);
                        }
                        ItemAction::ToggleTool(name) => {
                            let mut next = selected_tools.read().clone();
                            if next.contains(name) {
                                next.retain(|tool| tool != name);
                            } else {
                                next.push(name.clone());
                            }
                            selected_tools.set(next);
                        }
                        ItemAction::ToggleAdvanced => {
                            let was_shown = show_individual.get();
                            show_individual.set(!was_shown);
                            if was_shown {
                                let toggle =
                                    items.iter().position(|item| item.is_toggle).unwrap_or(0);
                                if focus_index.get() > toggle {
                                    focus_index.set(toggle);
                                }
                            }
                        }
                        ItemAction::Header => {}
                    }
                }
            }
        }
    }

    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    for (name, context, action) in [
        (
            "select:previous",
            ContextName::Select,
            InputAction::Previous,
        ),
        ("select:next", ContextName::Select, InputAction::Next),
        ("select:accept", ContextName::Select, InputAction::Accept),
        ("confirm:no", ContextName::Confirmation, InputAction::Cancel),
    ] {
        let mut pending_input = pending_input;
        use_keybinding(
            &mut hooks,
            runtime.clone(),
            name,
            context,
            || true,
            move || {
                pending_input.set(Some(action));
                true
            },
        );
    }

    let selected_set = valid_selected.iter().cloned().collect::<HashSet<_>>();
    let all_selected = !all_names.is_empty() && selected_set.len() == all_names.len();
    let rows = items.iter().enumerate().skip(1).map(|(index, item)| {
        let focused = index == focus_index.get();
        let divider = item.is_toggle.then(|| element! { Divider(width: Some(40u32)) });
        element! { Fragment {
            #(divider)
            #(item.is_header.then(|| element! { View(margin_top: 1u32) {} }))
            Text(
                key: item.id.clone(),
                content: if item.is_header { item.label.clone() } else if item.is_toggle { format!("{}[ {} ]", if focused { "❯ " } else { "  " }, item.label) } else { format!("{}{}", if focused { "❯ " } else { "  " }, item.label) },
                color: if !item.is_header && focused { Some(theme.suggestion) } else { None },
                dim: item.is_header,
                weight: if item.is_toggle && focused { Weight::Bold } else { Weight::Normal },
                wrap: TextWrap::NoWrap,
            )
        }}
    }).collect::<Vec<_>>();

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            Text(
                content: format!("{}[ Continue ]", if focus_index.get() == 0 { "❯ " } else { "  " }),
                color: if focus_index.get() == 0 { Some(theme.suggestion) } else { None },
                weight: if focus_index.get() == 0 { Weight::Bold } else { Weight::Normal },
            )
            Divider(width: Some(40u32))
            #(rows)
            View(margin_top: 1u32, flex_direction: FlexDirection::Column) {
                Text(content: if all_selected { "All tools selected".to_string() } else { format!("{} of {} tools selected", selected_set.len(), all_names.len()) }, dim: true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{StreamExt, stream};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn tools() -> Vec<AgentToolOption> {
        vec![
            "Read",
            "Edit",
            "Bash",
            "Custom",
            "mcp__github__issue",
            "Agent",
        ]
        .into_iter()
        .map(AgentToolOption::new)
        .collect()
    }

    #[test]
    fn bucket_and_advanced_rows_match_official_structure() {
        let available = tools()
            .into_iter()
            .filter(|tool| bucket_for(&tool.name).is_some())
            .collect::<Vec<_>>();
        let selected = available
            .iter()
            .map(|tool| tool.name.clone())
            .collect::<Vec<_>>();
        let compact = build_items(&available, &selected, false);
        assert_eq!(compact[0].label, "Continue");
        assert!(compact.iter().any(|item| item.label.contains("All tools")));
        assert!(
            compact
                .iter()
                .any(|item| item.label.contains("Read-only tools"))
        );
        assert!(compact.iter().any(|item| item.label.contains("MCP tools")));
        assert!(!compact.iter().any(|item| item.label == "MCP Servers:"));

        let advanced = build_items(&available, &selected, true);
        assert!(advanced.iter().any(|item| item.label == "MCP Servers:"));
        assert!(
            advanced
                .iter()
                .any(|item| item.label.contains("github (1 tool)"))
        );
        assert!(
            advanced
                .iter()
                .any(|item| item.label.contains("issue (github)"))
        );
        assert!(!advanced.iter().any(|item| item.label.contains(" Agent")));
    }

    #[test]
    fn action_keys_toggle_all_then_confirm_empty_selection() {
        let completed = Arc::new(Mutex::new(Vec::<Option<Vec<String>>>::new()));
        let completed_handler = Arc::clone(&completed);
        let completed_wait = Arc::clone(&completed);
        let events = stream::iter(vec![
            KeyCode::Down,
            KeyCode::Enter,
            KeyCode::Up,
            KeyCode::Enter,
        ])
        .then(|code| async move {
            futures_timer::Delay::new(Duration::from_millis(40)).await;
            TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, code))
        })
        .chain(stream::pending());
        futures::executor::block_on(async move {
            let mut app = element! {
                ContextProvider(value: Context::owned(
                    crate::keybindings::keybinding_context::KeybindingRuntime::with_default_bindings()
                )) {
                    ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                        ToolSelector(
                            tools: tools(),
                            on_complete: move |value| completed_handler.lock().expect("complete mutex").push(value),
                        )
                    }
                }
            };
            let mut loop_ = Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(100, 30),
            ));
            for _ in 0..40 {
                let _ = crate::utils::race(loop_.next(), async {
                    futures_timer::Delay::new(Duration::from_millis(80)).await;
                    None
                })
                .await;
                if !completed_wait.lock().expect("complete mutex").is_empty() {
                    break;
                }
            }
        });
        assert_eq!(
            *completed.lock().expect("complete mutex"),
            vec![Some(Vec::new())]
        );
    }
}
