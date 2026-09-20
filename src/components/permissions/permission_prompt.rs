//! Maps to: CC `components/permissions/PermissionPrompt.tsx`.

use crate::types::permissions::PermissionPromptChoice;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

const OPTION_CHOICES: &[PermissionPromptChoice] = &[
    PermissionPromptChoice::AllowOnce,
    PermissionPromptChoice::AlwaysAllow,
    PermissionPromptChoice::Deny,
];

#[derive(Default, Props)]
pub struct PermissionPromptProps {
    pub question: String,
    pub always_allow_label: String,
    pub on_select: Handler<PermissionPromptChoice>,
    pub on_cancel: Handler<()>,
}

#[component]
pub fn PermissionPrompt(
    props: &PermissionPromptProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let mut selected = hooks.use_state(|| 0usize);
    let runtime = hooks
        .try_use_context::<crate::keybindings::keybinding_context::KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    let handlers: crate::keybindings::use_keybinding::KeybindingHandlers = vec![
        (
            "select:previous".to_string(),
            Box::new(move || {
                selected.set(if selected.get() == 0 {
                    OPTION_CHOICES.len() - 1
                } else {
                    selected.get() - 1
                });
                true
            }),
        ),
        (
            "select:next".to_string(),
            Box::new(move || {
                selected.set((selected.get() + 1) % OPTION_CHOICES.len());
                true
            }),
        ),
        ("select:accept".to_string(), {
            let on_select = props.on_select.clone();
            Box::new(move || {
                (on_select)(OPTION_CHOICES[selected.get()]);
                true
            })
        }),
        ("select:cancel".to_string(), {
            let on_cancel = props.on_cancel.clone();
            Box::new(move || {
                (on_cancel)(());
                true
            })
        }),
    ];
    crate::keybindings::use_keybinding::use_keybindings(
        &mut hooks,
        runtime,
        handlers,
        crate::keybindings::types::ContextName::Select,
        || true,
    );

    element! {
        View(flex_direction: FlexDirection::Column, padding_left: 2u32, padding_right: 2u32) {
            Text(content: props.question.clone(), color: theme.permission, weight: Weight::Bold)
            #(OPTION_CHOICES.iter().enumerate().map(|(idx, choice)| {
                let focused = selected.get() == idx;
                let label = match choice {
                    PermissionPromptChoice::AllowOnce => "Yes".to_string(),
                    PermissionPromptChoice::AlwaysAllow => props.always_allow_label.clone(),
                    PermissionPromptChoice::Deny => "No".to_string(),
                };
                element! {
                    View(flex_direction: FlexDirection::Row, margin_top: if idx == 0 { 1u32 } else { 0u32 }) {
                        Text(
                            content: if focused { "❯ ".to_string() } else { "  ".to_string() },
                            color: if focused { theme.suggestion } else { theme.inactive },
                            wrap: TextWrap::NoWrap,
                        )
                        Text(
                            content: label,
                            color: if focused { theme.text } else { theme.inactive },
                            weight: if focused { Weight::Bold } else { Weight::Normal },
                            wrap: TextWrap::NoWrap,
                        )
                    }
                }
            }))
            View(margin_top: 1u32) {
                Text(content: "Esc to cancel".to_string(), color: theme.inactive)
            }
        }
    }
}
