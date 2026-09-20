//! Maps to: CC `components/SessionPreview.tsx`.
//! Read-only `/resume` preview for the main-screen local command UI selector.
//! Official Claude Code loads the full log with `loadFullLog()` and renders `Messages` in
//! transcript mode. This Rust port keeps the same ownership boundary but stays
//! native scrollback-safe: it reads the selected session, maps it through the
//! existing transcript mapper, and renders a bounded transcript preview without
//! writing session data or invoking tools.

use crate::commands::resume;
use crate::components::messages_list::Messages;
use crate::screens::repl::Screen;
use crate::types::message::RenderableMessage;
use crate::utils::conversation_recovery::renderable_messages_from_entries;
use crate::utils::format::format_relative_time_ago;
use crate::utils::session_storage::{SessionSelection, SessionSummary};
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::sync::Arc;

const FOOTER_LINES: usize = 3;
const MESSAGE_ROWS_ESTIMATE: usize = 3;
const PREVIEW_START_TAIL: usize = usize::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewNavigation {
    LineOlder,
    LineNewer,
    PageOlder,
    PageNewer,
    Oldest,
    Newest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SessionPreviewData {
    messages: Vec<RenderableMessage>,
    message_count: usize,
    error: Option<String>,
}

#[derive(Clone, Debug)]
struct PreviewWindow {
    messages: Vec<RenderableMessage>,
    start: usize,
    end: usize,
    total: usize,
}

#[derive(Default, Props)]
pub struct SessionPreviewProps<'a> {
    pub log: SessionSummary,
    pub max_height: usize,
    pub on_exit: HandlerMut<'a, ()>,
    pub on_select: HandlerMut<'a, SessionSelection>,
}

fn preview_limit(max_height: usize) -> usize {
    let available_rows = max_height.saturating_sub(FOOTER_LINES).max(1);
    (available_rows / MESSAGE_ROWS_ESTIMATE).max(1)
}

fn preview_window_size(max_height: usize, total_messages: usize) -> usize {
    preview_limit(max_height).min(total_messages.max(1)).max(1)
}

fn max_preview_start(total_messages: usize, max_height: usize) -> usize {
    total_messages.saturating_sub(preview_window_size(max_height, total_messages))
}

fn tail_preview_start(total_messages: usize, max_height: usize) -> usize {
    max_preview_start(total_messages, max_height)
}

fn clamp_preview_start(start: usize, total_messages: usize, max_height: usize) -> usize {
    start.min(max_preview_start(total_messages, max_height))
}

fn navigate_preview_start(
    current_start: usize,
    total_messages: usize,
    max_height: usize,
    navigation: PreviewNavigation,
) -> usize {
    let page = preview_window_size(max_height, total_messages);
    let max_start = max_preview_start(total_messages, max_height);
    match navigation {
        PreviewNavigation::LineOlder => current_start.saturating_sub(1),
        PreviewNavigation::LineNewer => current_start.saturating_add(1).min(max_start),
        PreviewNavigation::PageOlder => current_start.saturating_sub(page),
        PreviewNavigation::PageNewer => current_start.saturating_add(page).min(max_start),
        PreviewNavigation::Oldest => 0,
        PreviewNavigation::Newest => max_start,
    }
}

fn preview_navigation_for_key(
    code: &KeyCode,
    modifiers: &KeyModifiers,
) -> Option<PreviewNavigation> {
    let is_plain = !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
    match code {
        KeyCode::Up => Some(PreviewNavigation::LineOlder),
        KeyCode::Down => Some(PreviewNavigation::LineNewer),
        KeyCode::PageUp => Some(PreviewNavigation::PageOlder),
        KeyCode::PageDown => Some(PreviewNavigation::PageNewer),
        KeyCode::Home => Some(PreviewNavigation::Oldest),
        KeyCode::End => Some(PreviewNavigation::Newest),
        KeyCode::Char('k') if is_plain => Some(PreviewNavigation::LineOlder),
        KeyCode::Char('j') if is_plain => Some(PreviewNavigation::LineNewer),
        _ => None,
    }
}

fn build_preview_window(
    messages: &[RenderableMessage],
    max_height: usize,
    requested_start: usize,
) -> PreviewWindow {
    if messages.is_empty() {
        return PreviewWindow {
            messages: Vec::new(),
            start: 0,
            end: 0,
            total: 0,
        };
    }

    let window_size = preview_window_size(max_height, messages.len());
    let start = clamp_preview_start(requested_start, messages.len(), max_height);
    let end = start.saturating_add(window_size).min(messages.len());
    PreviewWindow {
        messages: messages[start..end].to_vec(),
        start,
        end,
        total: messages.len(),
    }
}

fn session_preview_footer_hint(has_navigation: bool) -> &'static str {
    if has_navigation {
        "Enter to resume · Esc to cancel · ↑/↓ to scroll · PgUp/PgDn to page"
    } else {
        "Enter to resume · Esc to cancel"
    }
}

fn preview_data_from_entries(
    entries: Vec<serde_json::Value>,
    _max_height: usize,
) -> SessionPreviewData {
    let messages = renderable_messages_from_entries(&entries);
    let message_count = messages.len();

    SessionPreviewData {
        messages,
        message_count,
        error: None,
    }
}

fn load_preview_data(log: &SessionSummary, max_height: usize) -> SessionPreviewData {
    let selection = SessionSelection::from(log);
    match resume::load_for_picker_selection(&selection) {
        Ok(target) => preview_data_from_entries(target.entries, max_height),
        Err(error) => SessionPreviewData {
            messages: Vec::new(),
            message_count: 0,
            error: Some(error.to_string()),
        },
    }
}

#[component]
pub fn SessionPreview<'a>(
    props: &mut SessionPreviewProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let root_height = props.max_height.saturating_sub(1).max(1) as u32;
    let preview_body_height = root_height.saturating_sub(FOOTER_LINES as u32).max(1);
    let deps = (
        props.log.session_id.clone(),
        props.log.file_path.display().to_string(),
        props.max_height,
    );
    let data = hooks.use_memo(
        || load_preview_data(&props.log, props.max_height),
        deps.clone(),
    );
    // Maps to: CC `SessionPreview.tsx:47-48` — "Get all base tools for preview
    // (no permissions needed for read-only view)", handed to `<Messages>` at
    // `:82`. CC calls `getAllBaseTools()` inline on every render; this port's
    // counterpart builds each tool's schema (and `agent_tool_schema` reloads the
    // agent definitions off disk), so the call is held for the preview's
    // lifetime instead.
    let tools: std::sync::Arc<Vec<crate::types::tools::Tool>> = hooks.use_memo(
        || std::sync::Arc::new(crate::tools::get_all_base_tools()),
        (),
    );
    let mut pending_exit = hooks.use_state(|| false);
    let mut pending_select = hooks.use_state(|| false);
    let runtime = hooks
        .try_use_context::<crate::keybindings::keybinding_context::KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    crate::keybindings::use_keybinding::use_keybinding(
        &mut hooks,
        runtime.clone(),
        "confirm:no",
        crate::keybindings::types::ContextName::Confirmation,
        || true,
        move || {
            pending_exit.set(true);
            true
        },
    );
    crate::keybindings::use_keybinding::use_keybinding(
        &mut hooks,
        runtime,
        "confirm:yes",
        crate::keybindings::types::ContextName::Confirmation,
        || true,
        move || {
            pending_select.set(true);
            true
        },
    );
    let mut preview_start = hooks.use_state(|| PREVIEW_START_TAIL);
    let mut preview_key = hooks.use_state(String::new);

    let preview_key_value = format!("{}:{}:{}", deps.0, deps.1, deps.2);
    let is_new_preview = { preview_key.read().as_str() != preview_key_value.as_str() };
    if is_new_preview {
        preview_key.set(preview_key_value);
        preview_start.set(PREVIEW_START_TAIL);
    }
    let requested_start = if is_new_preview {
        PREVIEW_START_TAIL
    } else {
        preview_start.get()
    };
    let effective_start = if requested_start == PREVIEW_START_TAIL {
        tail_preview_start(data.messages.len(), props.max_height)
    } else {
        clamp_preview_start(requested_start, data.messages.len(), props.max_height)
    };
    if !is_new_preview && requested_start != effective_start {
        preview_start.set(effective_start);
    }
    let preview_window = build_preview_window(&data.messages, props.max_height, effective_start);

    hooks.use_propagated_terminal_events({
        let current_start = preview_window.start;
        let total_messages = preview_window.total;
        let max_height = props.max_height;
        move |event| match event.event() {
            TerminalEvent::Key(KeyEvent {
                code,
                kind,
                modifiers,
                ..
            }) if *kind != KeyEventKind::Release => match code {
                // Maps to: CC SessionPreview uses configurable confirm:no/
                // confirm:yes for Escape/Enter; Ctrl-C remains the transcript
                // transport's raw cancellation fallback.
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    pending_exit.set(true);
                    event.stop_propagation();
                }
                _ => {
                    if let Some(navigation) = preview_navigation_for_key(code, modifiers) {
                        preview_start.set(navigate_preview_start(
                            current_start,
                            total_messages,
                            max_height,
                            navigation,
                        ));
                        event.stop_propagation();
                    }
                }
            },
            _ => {}
        }
    });

    if pending_exit.get() {
        pending_exit.set(false);
        (props.on_exit)(());
    }
    if pending_select.get() {
        pending_select.set(false);
        (props.on_select)(SessionSelection::from(&props.log));
    }

    let footer = {
        let mut parts = vec![
            format_relative_time_ago(props.log.modified),
            format!("{} messages", data.message_count),
        ];
        if let Some(branch) = props.log.git_branch.as_deref().filter(|s| !s.is_empty()) {
            parts.push(branch.to_string());
        }
        if preview_window.total > preview_window.messages.len() {
            parts.push(format!(
                "showing {}-{} of {}",
                preview_window.start.saturating_add(1),
                preview_window.end,
                preview_window.total
            ));
        }
        parts.join(" · ")
    };
    let preview_hint =
        session_preview_footer_hint(preview_window.total > preview_window.messages.len());

    element! {
        View(flex_direction: FlexDirection::Column, height: root_height, overflow: Overflow::Hidden) {
            View(height: preview_body_height, overflow: Overflow::Hidden, flex_shrink: 1.0f32) {
                #(if let Some(error) = data.error.clone() {
                    Some(element! {
                        View(padding_left: 2u32, padding_top: 1u32) {
                            Text(content: format!("Unable to preview session: {error}"), color: theme.error, wrap: TextWrap::Wrap)
                        }
                    }.into_any())
                } else {
                    Some(element! {
                        // Official SessionPreview renders Messages in transcript mode. The
                        // Rust main-screen port keeps native scrollback ownership by rendering
                        // a bounded, keyboard-navigable window over the full read-only preview.
                        Messages(
                            messages: Arc::new(preview_window.messages.clone()),
                            is_loading: false,
                            verbose: true,
                            screen: Screen::Transcript,
                            // CC `SessionPreview.tsx:82` `tools={tools}`.
                            tools: tools,
                        )
                    }.into_any())
                })
            }
            View(
                flex_shrink: 0.0f32,
                flex_direction: FlexDirection::Column,
                border_top_color: theme.inactive,
                border_style: BorderStyle::Single,
                border_bottom: false,
                border_left: false,
                border_right: false,
                padding_left: 2u32,
            ) {
                Text(content: footer, wrap: TextWrap::NoWrap)
                Text(content: preview_hint.to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preview_data_counts_message_entries_and_maps_transcript() {
        let entries = vec![
            json!({"type":"user","uuid":"u1","message":{"role":"user","content":"hello"}}),
            json!({"type":"assistant","uuid":"a1","message":{"role":"assistant","content":[{"type":"text","text":"world"}]}}),
            json!({"type":"tag","tag":"bug"}),
        ];

        let data = preview_data_from_entries(entries, 12);
        assert_eq!(data.message_count, 2);
        assert_eq!(data.messages.len(), 2);
        assert!(data.error.is_none());
    }

    #[test]
    fn preview_window_defaults_to_tail_when_bounded() {
        let messages = (0..10)
            .map(|idx| RenderableMessage::user(format!("m{idx}"), format!("message {idx}")))
            .collect::<Vec<_>>();

        let start = tail_preview_start(messages.len(), 12);
        let window = build_preview_window(&messages, 12, start);
        assert_eq!(window.start, 7);
        assert_eq!(window.end, 10);
        assert_eq!(window.messages[0].uuid, "m7");
    }

    #[test]
    fn preview_navigation_moves_and_clamps_within_full_transcript() {
        assert_eq!(
            navigate_preview_start(7, 10, 12, PreviewNavigation::LineOlder),
            6
        );
        assert_eq!(
            navigate_preview_start(7, 10, 12, PreviewNavigation::LineNewer),
            7
        );
        assert_eq!(
            navigate_preview_start(7, 10, 12, PreviewNavigation::PageOlder),
            4
        );
        assert_eq!(
            navigate_preview_start(4, 10, 12, PreviewNavigation::PageNewer),
            7
        );
        assert_eq!(
            navigate_preview_start(4, 10, 12, PreviewNavigation::Oldest),
            0
        );
        assert_eq!(
            navigate_preview_start(4, 10, 12, PreviewNavigation::Newest),
            7
        );
    }

    #[test]
    fn preview_footer_hint_matches_official_byline_copy() {
        assert_eq!(
            session_preview_footer_hint(false),
            "Enter to resume · Esc to cancel"
        );
        assert_eq!(
            session_preview_footer_hint(true),
            "Enter to resume · Esc to cancel · ↑/↓ to scroll · PgUp/PgDn to page"
        );
    }

    #[test]
    fn preview_navigation_keymap_matches_session_preview_controls() {
        assert_eq!(
            preview_navigation_for_key(&KeyCode::PageUp, &KeyModifiers::empty()),
            Some(PreviewNavigation::PageOlder)
        );
        assert_eq!(
            preview_navigation_for_key(&KeyCode::Char('k'), &KeyModifiers::empty()),
            Some(PreviewNavigation::LineOlder)
        );
        assert_eq!(
            preview_navigation_for_key(&KeyCode::Char('k'), &KeyModifiers::CONTROL),
            None
        );
    }
}
