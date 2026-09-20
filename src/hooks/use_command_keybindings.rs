//! Maps to: CC `hooks/useCommandKeybindings.tsx`.
//!
//! The official null-rendering component registers every configured
//! `command:*` action and forwards `/<name>` to REPL `onSubmit` with
//! `fromKeybinding: true`. iocraft handlers cannot capture a borrowed
//! `HandlerMut`, so this hook records a pending command; REPL consumes it on
//! the next retained update and invokes its normal submit owner. PromptInput is
//! not cleared, preserving the official keybinding behavior.

use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::{KeybindingHandler, KeybindingHandlers, use_keybindings};
use iocraft::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct CommandKeybindingHandlersState {
    pending_command: State<Option<String>>,
}

impl CommandKeybindingHandlersState {
    pub fn take_pending_command(&mut self) -> Option<String> {
        let command = self.pending_command.read().clone();
        if command.is_some() {
            self.pending_command.set(None);
        }
        command
    }
}

/// Maps to: CC `CommandKeybindingHandlers`.
pub fn use_command_keybinding_handlers(
    hooks: &mut Hooks,
    runtime: Option<KeybindingRuntime>,
    is_active: bool,
) -> CommandKeybindingHandlersState {
    let empty_bindings =
        hooks.use_const(Arc::<Vec<crate::keybindings::types::ParsedBinding>>::default);
    let bindings = runtime
        .as_ref()
        .map(KeybindingRuntime::bindings)
        .unwrap_or(empty_bindings);
    let bindings_identity = Arc::as_ptr(&bindings) as usize;
    let command_actions: Arc<Vec<String>> = hooks.use_memo(
        {
            let bindings = Arc::clone(&bindings);
            move || {
                let mut seen = HashSet::new();
                Arc::new(
                    bindings
                        .iter()
                        .filter_map(|binding| binding.action.as_ref())
                        .filter(|action| action.starts_with("command:"))
                        .filter(|action| seen.insert((*action).clone()))
                        .cloned()
                        .collect(),
                )
            }
        },
        bindings_identity,
    );

    let pending_command = hooks.use_state(|| Option::<String>::None);
    let handlers: KeybindingHandlers = command_actions
        .iter()
        .map(|action| {
            let command = format!("/{}", &action["command:".len()..]);
            let mut pending_command = pending_command;
            let handler: KeybindingHandler = Box::new(move || {
                pending_command.set(Some(command.clone()));
                true
            });
            (action.clone(), handler)
        })
        .collect();
    use_keybindings(hooks, runtime, handlers, ContextName::Chat, move || {
        is_active
    });

    CommandKeybindingHandlersState { pending_command }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{StreamExt, stream};
    use std::time::Duration;

    #[derive(Default, Props)]
    struct CommandKeybindingHarnessProps {
        active: bool,
    }

    #[component]
    fn CommandKeybindingHarness(
        props: &CommandKeybindingHarnessProps,
        mut hooks: Hooks,
    ) -> impl Into<AnyElement<'static>> {
        let runtime = hooks.use_const(|| {
            KeybindingRuntime::new(vec![crate::keybindings::types::ParsedBinding {
                chord: crate::keybindings::parser::parse_chord("ctrl+j"),
                action: Some("command:cost".to_string()),
                context: ContextName::Chat,
            }])
        });
        let mut state = use_command_keybinding_handlers(&mut hooks, Some(runtime), props.active);
        let mut submitted = hooks.use_state(String::new);
        if let Some(command) = state.take_pending_command() {
            submitted.set(command);
        }
        element! { Text(content: submitted.read().clone()) }
    }

    #[test]
    fn command_binding_handler_projects_dynamic_action_to_slash_submit_text() {
        let mut event = KeyEvent::new(KeyEventKind::Press, KeyCode::Char('j'));
        event.modifiers = KeyModifiers::CONTROL;
        // mock_terminal_render_loop does not end when the event stream ends.
        let text = futures::executor::block_on(async move {
            let mut app = element!(CommandKeybindingHarness(active: true));
            let mut render_loop = Box::pin(
                app.mock_terminal_render_loop(
                    MockTerminalConfig::with_events(stream::iter(vec![TerminalEvent::Key(event)]))
                        .with_size(30, 4),
                ),
            );
            let mut last = String::new();
            for _ in 0..16 {
                let next = crate::utils::race(render_loop.next(), async {
                    futures_timer::Delay::new(Duration::from_millis(100)).await;
                    None
                })
                .await;
                let Some(canvas) = next else {
                    break;
                };
                last = canvas.to_string();
                if last.contains("/cost") {
                    break;
                }
            }
            last
        });
        assert!(text.contains("/cost"), "canvas=\n{text}");
    }

    #[test]
    fn command_binding_is_disabled_while_modal_overlay_is_active() {
        let mut event = KeyEvent::new(KeyEventKind::Press, KeyCode::Char('j'));
        event.modifiers = KeyModifiers::CONTROL;
        let text = futures::executor::block_on(async move {
            let mut app = element!(CommandKeybindingHarness(active: false));
            let mut render_loop = Box::pin(
                app.mock_terminal_render_loop(
                    MockTerminalConfig::with_events(stream::iter(vec![TerminalEvent::Key(event)]))
                        .with_size(30, 4),
                ),
            );
            let mut last = String::new();
            for _ in 0..8 {
                let next = crate::utils::race(render_loop.next(), async {
                    futures_timer::Delay::new(Duration::from_millis(100)).await;
                    None
                })
                .await;
                let Some(canvas) = next else {
                    break;
                };
                last = canvas.to_string();
            }
            last
        });
        assert!(!text.contains("/cost"), "canvas=\n{text}");
    }
}
