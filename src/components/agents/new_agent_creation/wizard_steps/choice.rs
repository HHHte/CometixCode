use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use iocraft::prelude::*;

#[derive(Clone, Copy)]
enum Action {
    Previous,
    Next,
    Accept,
    Cancel,
}

#[derive(Default, Props)]
pub struct WizardChoiceProps<'a> {
    pub options: Vec<SelectOptionData>,
    pub on_select: HandlerMut<'a, String>,
    pub on_cancel: HandlerMut<'a, ()>,
}

#[component]
pub fn WizardChoice<'a>(
    props: &mut WizardChoiceProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut focused = hooks.use_state(|| 0usize);
    let mut pending = hooks.use_state(|| None::<Action>);
    let action = { *pending.read() };
    if let Some(action) = action {
        pending.set(None);
        match action {
            Action::Previous if !props.options.is_empty() => focused.set(if focused.get() == 0 {
                props.options.len() - 1
            } else {
                focused.get() - 1
            }),
            Action::Next if !props.options.is_empty() => {
                focused.set((focused.get() + 1) % props.options.len())
            }
            Action::Accept => {
                if let Some(option) = props.options.get(focused.get()) {
                    (props.on_select)(option.value.clone());
                }
            }
            Action::Cancel => (props.on_cancel)(()),
            _ => {}
        }
    }
    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|value| value.clone());
    for (name, context, action) in [
        ("select:previous", ContextName::Select, Action::Previous),
        ("select:next", ContextName::Select, Action::Next),
        ("select:accept", ContextName::Select, Action::Accept),
        ("confirm:no", ContextName::Confirmation, Action::Cancel),
    ] {
        let mut pending = pending;
        use_keybinding(
            &mut hooks,
            runtime.clone(),
            name,
            context,
            || true,
            move || {
                pending.set(Some(action));
                true
            },
        );
    }
    element! { Select(options: props.options.clone(), focused_index: focused.get(), visible_option_count: props.options.len(), layout: SelectLayout::Expanded) }
}
