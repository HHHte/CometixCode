//! Maps to: CC `components/agents/ColorPicker.tsx:1-106`.

use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use crate::tools::agent_tool::agent_color_manager::{AGENT_COLORS, AgentColorName};
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorOption {
    Automatic,
    Color(AgentColorName),
}

fn color_options() -> Vec<ColorOption> {
    std::iter::once(ColorOption::Automatic)
        .chain(AGENT_COLORS.into_iter().map(ColorOption::Color))
        .collect()
}

fn color_label(color: AgentColorName) -> String {
    let value = color.official_name();
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

#[derive(Clone, Copy, Debug)]
enum PickerAction {
    Previous,
    Next,
    Confirm,
}

#[derive(Default, Props)]
pub struct ColorPickerProps<'a> {
    pub agent_name: String,
    pub current_color: Option<AgentColorName>,
    pub on_confirm: HandlerMut<'a, Option<AgentColorName>>,
}

#[component]
pub fn ColorPicker<'a>(
    props: &mut ColorPickerProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let options = color_options();
    let initial = props
        .current_color
        .and_then(|color| {
            options
                .iter()
                .position(|option| *option == ColorOption::Color(color))
        })
        .unwrap_or(0);
    let mut selected_index = hooks.use_state(move || initial);
    let mut pending_action = hooks.use_state(|| None::<PickerAction>);
    let mut pending_confirm = hooks.use_state(|| None::<Option<AgentColorName>>);

    let confirm = {
        let value = pending_confirm.read();
        *value
    };
    if let Some(color) = confirm {
        pending_confirm.set(None);
        (props.on_confirm)(color);
    }
    let action = {
        let value = pending_action.read();
        *value
    };
    if let Some(action) = action {
        pending_action.set(None);
        match action {
            PickerAction::Previous => selected_index.set(if selected_index.get() == 0 {
                options.len() - 1
            } else {
                selected_index.get() - 1
            }),
            PickerAction::Next => selected_index.set((selected_index.get() + 1) % options.len()),
            PickerAction::Confirm => {
                let value = match options[selected_index.get()] {
                    ColorOption::Automatic => None,
                    ColorOption::Color(color) => Some(color),
                };
                // Outer Some marks a pending callback; inner Option preserves automatic.
                pending_confirm.set(Some(value));
            }
        }
    }

    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    for (name, action) in [
        ("select:previous", PickerAction::Previous),
        ("select:next", PickerAction::Next),
        ("select:accept", PickerAction::Confirm),
    ] {
        let mut pending_action = pending_action;
        use_keybinding(
            &mut hooks,
            runtime.clone(),
            name,
            ContextName::Select,
            || true,
            move || {
                pending_action.set(Some(action));
                true
            },
        );
    }

    let selected = options[selected_index.get()];
    let rows = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let is_selected = index == selected_index.get();
            let label = match option {
                ColorOption::Automatic => "Automatic color".to_string(),
                ColorOption::Color(color) => color_label(*color),
            };
            let swatch = match option {
                ColorOption::Automatic => None,
                ColorOption::Color(color) => Some(
                    crate::utils::iocraft_color::to_iocraft_color(
                        Some(color.official_name()),
                        *theme,
                    ),
                ),
            };
            element! {
                View(flex_direction: FlexDirection::Row, column_gap: 1u32) {
                    Text(
                        content: if is_selected { "❯".to_string() } else { " ".to_string() },
                        color: if is_selected { Some(theme.suggestion) } else { None },
                        wrap: TextWrap::NoWrap,
                    )
                    #(swatch.map(|background| element! {
                        Text(content: " ".to_string(), background_color: Some(background), color: theme.inverse_text, wrap: TextWrap::NoWrap)
                    }))
                    Text(content: label, weight: if is_selected { Weight::Bold } else { Weight::Normal }, wrap: TextWrap::NoWrap)
                }
            }
        })
        .collect::<Vec<_>>();
    let preview = format!(" @{} ", props.agent_name);
    let preview_element = match selected {
        ColorOption::Automatic => element! {
            Text(content: preview, background_color: Some(theme.text), color: theme.background, weight: Weight::Bold, wrap: TextWrap::NoWrap)
        }.into_any(),
        ColorOption::Color(color) => element! {
            Text(content: preview, background_color: Some(crate::utils::iocraft_color::to_iocraft_color(Some(color.official_name()), *theme)), color: theme.inverse_text, weight: Weight::Bold, wrap: TextWrap::NoWrap)
        }.into_any(),
    };

    element! {
        View(flex_direction: FlexDirection::Column, row_gap: 1u32) {
            View(flex_direction: FlexDirection::Column) { #(rows) }
            View(flex_direction: FlexDirection::Row, margin_top: 1u32) {
                Text(content: "Preview: ".to_string())
                #(vec![preview_element])
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

    #[test]
    fn picker_renders_automatic_all_colors_and_preview() {
        let text = element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                ColorPicker(agent_name: "reviewer".to_string())
            }
        }
        .render(Some(60))
        .to_string();
        assert!(text.contains("❯ Automatic color"));
        for label in [
            "Red", "Blue", "Green", "Yellow", "Purple", "Orange", "Pink", "Cyan",
        ] {
            assert!(text.contains(label), "missing {label}; canvas=\n{text}");
        }
        assert!(text.contains("Preview:  @reviewer"));
    }

    #[test]
    fn action_navigation_confirms_selected_color() {
        let confirmed = Arc::new(Mutex::new(Vec::<Option<AgentColorName>>::new()));
        let confirmed_handler = Arc::clone(&confirmed);
        let confirmed_wait = Arc::clone(&confirmed);
        let events = stream::iter(vec![KeyCode::Down, KeyCode::Enter])
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
                        ColorPicker(
                            agent_name: "reviewer".to_string(),
                            on_confirm: move |color| confirmed_handler.lock().expect("confirm mutex").push(color),
                        )
                    }
                }
            };
            let mut loop_ = Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(80, 24),
            ));
            for _ in 0..25 {
                let _ = crate::utils::race(loop_.next(), async {
                    futures_timer::Delay::new(Duration::from_millis(80)).await;
                    None
                })
                .await;
                if !confirmed_wait.lock().expect("confirm mutex").is_empty() {
                    break;
                }
            }
        });
        assert_eq!(
            *confirmed.lock().expect("confirm mutex"),
            vec![Some(AgentColorName::Red)]
        );
    }
}
