//! Maps to: CC `commands/hooks/hooks.tsx` `call`.

use crate::components::hooks::{HooksConfigMenu, HooksConfigMenuDone};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct HooksCommandProps<'a> {
    pub tool_names: Vec<String>,
    pub on_done: HandlerMut<'a, HooksConfigMenuDone>,
}

/// Rust component boundary for the official lazy local-JSX command call.
#[component]
pub fn HooksCommand<'a>(
    props: &mut HooksCommandProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut pending_done = hooks.use_state(|| Option::<HooksConfigMenuDone>::None);
    let done = { pending_done.read().clone() };
    if let Some(done) = done {
        pending_done.set(None);
        (props.on_done)(done);
    }
    let mut pending_done_for_menu = pending_done;

    element! {
        HooksConfigMenu(
            tool_names: props.tool_names.clone(),
            on_exit: move |done| pending_done_for_menu.set(Some(done)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::hooks::hooks_config_menu::HooksConfigMenuOverride;
    use crate::utils::theme;

    #[test]
    fn hooks_command_call_renders_official_menu_boundary() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ContextProvider(value: Context::owned(HooksConfigMenuOverride::default())) {
                    HooksCommand(tool_names: vec!["Bash".to_string()])
                }
            }
        }
        .render(Some(100))
        .to_string();
        assert!(text.contains("Hooks"), "canvas=\n{text}");
        assert!(text.contains("This menu is read-only"), "canvas=\n{text}");
    }
}
