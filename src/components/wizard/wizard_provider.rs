//! Maps to: CC `components/wizard/WizardProvider.tsx:1-131`.

use super::types::{
    WizardAction, WizardContextValue, WizardMachine, WizardProviderProps, WizardTransition,
};
use iocraft::prelude::*;
use std::sync::Arc;

/// Owns wizard step/data/history state and injects `WizardContextValue`.
#[component]
pub fn WizardProvider<'a>(
    props: &mut WizardProviderProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let steps_len = props.steps.len();
    let initial_data = props.initial_data.clone();
    let mut machine = hooks.use_state(move || WizardMachine::new(steps_len, initial_data));
    let mut pending_complete = hooks.use_state(|| None);
    let mut pending_cancel = hooks.use_state(|| false);
    let channel = hooks.use_const(|| Arc::new(async_channel::unbounded::<WizardAction>()));
    let action_sender = channel.0.clone();
    let action_receiver = channel.1.clone();

    // Context actions cross the retained component boundary through a channel;
    // the provider remains the sole owner of hook state, like React setState.
    hooks.use_future(async move {
        while let Ok(action) = action_receiver.recv().await {
            let mut next = machine.read().clone();
            let transition = match action {
                WizardAction::GoNext => next.go_next(),
                WizardAction::GoBack => next.go_back(),
                WizardAction::GoToStep(index) => next.go_to_step(index),
                WizardAction::Cancel => next.cancel(),
                WizardAction::SetData(data) => {
                    next.wizard_data = data;
                    WizardTransition::Changed
                }
                WizardAction::UpdateData(updates) => {
                    next.update_wizard_data(updates);
                    WizardTransition::Changed
                }
            };
            match transition {
                WizardTransition::Complete(data) => pending_complete.set(Some(data)),
                WizardTransition::Cancel => pending_cancel.set(true),
                WizardTransition::None | WizardTransition::Changed => {}
            }
            machine.set(next);
        }
    });

    let _ = crate::hooks::use_exit::use_exit_on_ctrl_cd_with_keybindings(&mut hooks, true);

    let completion = { pending_complete.read().clone() };
    if let Some(data) = completion {
        pending_complete.set(None);
        (props.on_complete)(data);
    }
    if pending_cancel.get() {
        pending_cancel.set(false);
        (props.on_cancel)(());
    }

    let snapshot = machine.read().clone();
    if snapshot.is_completed || snapshot.current_step_index >= props.steps.len() {
        return element! { Fragment }.into_any();
    }

    let context = WizardContextValue {
        current_step_index: snapshot.current_step_index,
        total_steps: snapshot.total_steps,
        wizard_data: snapshot.wizard_data,
        title: props.title.clone(),
        show_step_counter: props.show_step_counter.unwrap_or(true),
        action_sender,
    };
    let content = props
        .child_renderer
        .as_ref()
        .unwrap_or(&props.steps[snapshot.current_step_index])
        .render();

    element! {
        ContextProvider(value: Context::owned(context)) {
            #(vec![content])
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::wizard::types::WizardData;
    use crate::components::wizard::use_wizard::use_wizard;
    use crate::components::wizard::wizard_dialog_layout::WizardDialogLayout;
    use futures::{StreamExt, stream};
    use std::sync::Mutex;
    use std::time::Duration;

    #[derive(Default, Props)]
    struct TestStepProps {
        label: String,
    }

    #[component]
    fn TestStep(props: &TestStepProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
        let wizard = use_wizard(&mut hooks);
        let runtime = hooks
            .try_use_context::<crate::keybindings::keybinding_context::KeybindingRuntime>()
            .map(|runtime| runtime.clone());
        crate::keybindings::use_keybinding::use_keybinding(
            &mut hooks,
            runtime.clone(),
            "confirm:yes",
            crate::keybindings::types::ContextName::Confirmation,
            || true,
            {
                let wizard = wizard.clone();
                move || {
                    wizard.go_next();
                    true
                }
            },
        );
        crate::keybindings::use_keybinding::use_keybinding(
            &mut hooks,
            runtime,
            "confirm:no",
            crate::keybindings::types::ContextName::Confirmation,
            || true,
            {
                let wizard = wizard.clone();
                move || {
                    wizard.go_back();
                    true
                }
            },
        );
        element! {
            WizardDialogLayout(subtitle: Some(props.label.clone())) {
                Text(content: format!("Body {}", props.label))
            }
        }
    }

    fn test_steps() -> Vec<super::super::types::WizardStep> {
        vec![
            super::super::types::WizardStep::new(|| {
                element! { TestStep(label: "First".to_string()) }.into_any()
            }),
            super::super::types::WizardStep::new(|| {
                element! { TestStep(label: "Second".to_string()) }.into_any()
            }),
        ]
    }

    #[test]
    fn provider_renders_current_step_context_and_layout() {
        let theme = *crate::utils::theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(theme)) {
                WizardProvider(
                    steps: test_steps(),
                    initial_data: WizardData::new(),
                    title: Some("Create new agent".to_string()),
                )
            }
        }
        .render(Some(100));
        let text = canvas.to_string();
        assert!(text.contains("Create new agent (1/2)"), "canvas=\n{text}");
        assert!(text.contains("First"), "canvas=\n{text}");
        assert!(text.contains("Body First"), "canvas=\n{text}");
        assert!(
            text.contains("↑↓ to navigate · Enter to select · Esc to go back"),
            "canvas=\n{text}"
        );
        let (y, line) = text
            .lines()
            .enumerate()
            .find(|(_, line)| line.contains("Create new agent"))
            .expect("title");
        let x = line.find("Create").unwrap();
        assert_eq!(
            canvas.resolved_text_style(x, y).unwrap().color,
            Some(theme.suggestion)
        );
    }

    #[test]
    fn provider_action_key_completion_returns_final_data_once() {
        let completed = Arc::new(Mutex::new(Vec::<WizardData>::new()));
        let completed_for_handler = Arc::clone(&completed);
        let completed_for_wait = Arc::clone(&completed);
        let events = stream::iter(vec![KeyCode::Enter, KeyCode::Enter]).then(|code| async move {
            futures_timer::Delay::new(Duration::from_millis(45)).await;
            TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, code))
        });
        let mut initial_data = WizardData::new();
        initial_data.insert("name".to_string(), serde_json::json!("demo"));

        futures::executor::block_on(async move {
            let mut app = element! {
                ContextProvider(value: Context::owned(
                    crate::keybindings::keybinding_context::KeybindingRuntime::with_default_bindings()
                )) {
                    ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                        WizardProvider(
                            steps: test_steps(),
                            initial_data: initial_data,
                            on_complete: move |data| completed_for_handler.lock().expect("complete mutex").push(data),
                        )
                    }
                }
            };
            let mut loop_ = Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(100, 24),
            ));
            for _ in 0..30 {
                let next = crate::utils::race(loop_.next(), async {
                    futures_timer::Delay::new(Duration::from_millis(80)).await;
                    None
                })
                .await;
                if next.is_none()
                    || !completed_for_wait
                        .lock()
                        .expect("complete mutex")
                        .is_empty()
                {
                    break;
                }
            }
        });
        let values = completed.lock().expect("complete mutex");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0]["name"], serde_json::json!("demo"));
    }

    #[test]
    fn provider_action_key_navigation_backs_out_and_cancels() {
        let cancelled = Arc::new(Mutex::new(0usize));
        let cancelled_for_handler = Arc::clone(&cancelled);
        let cancelled_for_wait = Arc::clone(&cancelled);
        let events = vec![KeyCode::Enter, KeyCode::Esc, KeyCode::Esc];
        let events = stream::iter(events).then(|code| async move {
            futures_timer::Delay::new(Duration::from_millis(45)).await;
            TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, code))
        });

        futures::executor::block_on(async move {
            let mut app = element! {
                ContextProvider(value: Context::owned(
                    crate::keybindings::keybinding_context::KeybindingRuntime::with_default_bindings()
                )) {
                    ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                        WizardProvider(
                            steps: test_steps(),
                            initial_data: WizardData::new(),
                            title: Some("Test wizard".to_string()),
                            on_cancel: move |_| *cancelled_for_handler.lock().expect("cancel mutex") += 1,
                        )
                    }
                }
            };
            let mut loop_ = Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(100, 24),
            ));
            for _ in 0..40 {
                let next = crate::utils::race(loop_.next(), async {
                    futures_timer::Delay::new(Duration::from_millis(80)).await;
                    None
                })
                .await;
                if next.is_none() || *cancelled_for_wait.lock().expect("cancel mutex") > 0 {
                    break;
                }
            }
        });
        assert_eq!(*cancelled.lock().expect("cancel mutex"), 1);
    }
}
