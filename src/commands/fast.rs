//! Maps to: CC `commands/fast/fast.tsx#FastModePicker`.
//!
//! The picker owns only interactive selection. AppState/model mutation and API
//! request options remain with PromptInput/REPL runtime owners.

use crate::components::design_system::dialog::Dialog;
use crate::components::fast_icon::FastIcon;
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct FastModePickerProps<'a> {
    pub initial_enabled: bool,
    pub unavailable_reason: Option<String>,
    pub on_done: HandlerMut<'a, Option<bool>>,
}

#[component]
pub fn FastModePicker<'a>(
    props: &mut FastModePickerProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut enabled = hooks.use_state(|| props.initial_enabled);
    let mut pending = hooks.use_state(|| Option::<Option<bool>>::None);
    let unavailable = props.unavailable_reason.is_some();
    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    for action in [
        "confirm:nextField",
        "confirm:next",
        "confirm:previous",
        "confirm:cycleMode",
        "confirm:toggle",
    ] {
        use_keybinding(
            &mut hooks,
            runtime.clone(),
            action,
            ContextName::Confirmation,
            move || !unavailable,
            move || {
                enabled.set(!enabled.get());
                true
            },
        );
    }
    use_keybinding(
        &mut hooks,
        runtime,
        "confirm:yes",
        ContextName::Confirmation,
        move || !unavailable,
        move || {
            pending.set(Some(Some(enabled.get())));
            true
        },
    );
    let completed = { *pending.read() };
    if let Some(result) = completed {
        pending.set(None);
        (props.on_done)(result);
    }
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let input_guide = if unavailable {
        "Esc to cancel".to_string()
    } else {
        "Tab to toggle · Enter to confirm · Esc to cancel".to_string()
    };
    element! {
        Dialog(
            title_children: vec![element! {
                View(flex_direction: FlexDirection::Row) {
                    FastIcon
                    Text(content: " Fast mode (research preview)".to_string(), weight: Weight::Bold)
                }
            }.into_any()],
            subtitle: Some(format!(
            "High-speed mode for {}. Billed as extra usage at a premium rate. Separate rate limits apply.",
            crate::utils::fast_mode::FAST_MODE_MODEL_DISPLAY,
        )),
            color: Some(theme.fast_mode),
            input_guide: Some(input_guide),
            on_cancel: move |_| pending.set(Some(None)),
        ) {
            #(if let Some(reason) = props.unavailable_reason.clone() {
                element! { View(margin_left: 2u32) { Text(content: reason, color: theme.error) } }.into_any()
            } else {
                element! {
                    View(flex_direction: FlexDirection::Row, column_gap: 2u32, margin_left: 2u32) {
                        Text(content: "Fast mode".to_string(), weight: Weight::Bold)
                        Text(
                            content: if enabled.get() { "ON" } else { "OFF" }.to_string(),
                            color: if enabled.get() { theme.fast_mode } else { theme.text },
                            weight: if enabled.get() { Weight::Bold } else { Weight::Normal },
                        )
                    }
                }.into_any()
            })
            View(flex_direction: FlexDirection::Row) {
                Text(content: "Learn more: ".to_string(), dim: true)
                Link(url: "https://code.claude.com/docs/en/fast-mode".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_mode_picker_renders_official_external_copy() {
        let text = element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                FastModePicker(initial_enabled: false)
            }
        }
        .render(Some(100))
        .to_string();
        assert!(
            text.contains("Fast mode (research preview)"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Fast mode  OFF"), "canvas=\n{text}");
        assert!(
            text.contains("High-speed mode for Opus 4.6"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Tab to toggle"), "canvas=\n{text}");
    }
}
