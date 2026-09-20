//! `/permissions` command adapter.
//! Maps to: CC `commands/permissions/permissions.tsx`.
//!
//! The command owner only mounts the canonical permission-rules component and
//! adapts its exit callback to the REPL local-command boundary. Permission rule
//! rendering and interaction remain owned by
//! `components/permissions/rules/permission_rule_list.rs`.

use crate::components::permissions::rules::permission_rule_list::{
    PermissionRuleList, PermissionRuleListExit,
};
use crate::tool::ToolPermissionContext;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct PermissionsCommandPanelProps<'a> {
    pub context: ToolPermissionContext,
    pub on_result: HandlerMut<'a, PermissionRuleListExit>,
    /// CC context.setMessages: append the canonical retry system message.
    pub on_message: HandlerMut<'a, crate::types::message::Message>,
}

/// Native ordered callback inbox for CC onRetryDenials → onExit.
#[derive(Clone)]
enum PermissionsCommandEvent {
    Message(crate::types::message::Message),
    Done(PermissionRuleListExit),
}

/// Maps to: CC `commands/permissions/permissions.tsx` `call(...)`, which mounts
/// `PermissionRuleList` and delegates completion to the command callback.
#[component]
pub fn PermissionsCommandPanel<'a>(
    props: &mut PermissionsCommandPanelProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    // PORTING.md React/Ink callback-delivery mapping: WorkspaceTab's `'static`
    // Select handler records PermissionRuleList.onExit, then this command owner invokes the
    // borrowed local-command callback exactly once on the following frame.
    let mut pending = hooks.use_state(Vec::<PermissionsCommandEvent>::new);
    let completed = { pending.read().clone() };
    if !completed.is_empty() {
        pending.set(Vec::new());
        for event in completed {
            match event {
                PermissionsCommandEvent::Message(message) => (props.on_message)(message),
                PermissionsCommandEvent::Done(result) => (props.on_result)(result),
            }
        }
    }
    element! {
        PermissionRuleList(
            context: props.context.clone(),
            on_retry_denials: move |commands: Vec<String>| {
                let mut events = pending.read().clone();
                events.push(PermissionsCommandEvent::Message(crate::types::message::Message::System(
                    crate::utils::messages::create_permission_retry_message(commands),
                )));
                pending.set(events);
            },
            on_exit: move |result: PermissionRuleListExit| {
                let mut events = pending.read().clone();
                events.push(PermissionsCommandEvent::Done(result));
                pending.set(events);
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::{PermissionRuleSource, PermissionRuleValue};
    use crate::utils::theme;

    #[test]
    fn permissions_command_mounts_canonical_permission_rule_list() {
        let mut context = ToolPermissionContext::default();
        context.always_allow_rules.insert(
            PermissionRuleSource::Session,
            vec![PermissionRuleValue::new(
                "Bash",
                Some("git status".to_string()),
            )],
        );

        let text = element! {
            ContextProvider(value: Context::owned(
                crate::keybindings::keybinding_context::KeybindingRuntime::with_default_bindings()
            )) {
                ContextProvider(value: Context::owned(*theme::current())) {
                    PermissionsCommandPanel(context: context)
                }
            }
        }
        .render(Some(120))
        .to_string();

        assert!(text.contains("Permissions:"), "canvas=\n{text}");
        assert!(text.contains("Bash(git status)"), "canvas=\n{text}");
        assert!(
            !text.contains("From current session"),
            "source list has no source description; canvas=\n{text}"
        );
    }
}
