//! Maps to: CC `commands/diff/diff.tsx:1-7` `call`.
//! The official call lazily returns `DiffDialog`; this component preserves that
//! command boundary while REPL continues to own local-command completion.

use crate::components::diff::{DiffDialog, DiffDialogDone};
use crate::types::message::Message;
use iocraft::prelude::*;
use std::sync::Arc;

#[derive(Default, Props)]
pub struct DiffCommandProps<'a> {
    pub messages: Arc<Vec<Message>>,
    pub on_done: HandlerMut<'a, DiffDialogDone>,
}

#[component]
pub fn DiffCommand<'a>(
    props: &mut DiffCommandProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut pending_done = hooks.use_state(|| Option::<DiffDialogDone>::None);
    let done = { pending_done.read().clone() };
    if let Some(done) = done {
        pending_done.set(None);
        (props.on_done)(done);
    }
    let mut pending_for_dialog = pending_done;

    element! {
        DiffDialog(
            messages: Arc::clone(&props.messages),
            on_done: move |done| pending_for_dialog.set(Some(done)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::use_diff_data::{DiffData, DiffDataOverride};
    use crate::utils::theme;

    #[test]
    fn diff_command_call_renders_official_diff_dialog_boundary() {
        let current_theme = *theme::current();
        let text = element! {
            ContextProvider(value: Context::owned(current_theme)) {
                ContextProvider(value: Context::owned(DiffDataOverride(DiffData::default()))) {
                    DiffCommand(messages: Arc::new(Vec::new()))
                }
            }
        }
        .render(Some(80))
        .to_string();
        assert!(text.contains("Uncommitted changes (git diff HEAD)"));
        assert!(text.contains("Working tree is clean"));
    }
}
