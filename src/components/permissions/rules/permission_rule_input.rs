//! Maps to: CC `components/permissions/rules/PermissionRuleInput.tsx`.

use crate::components::design_system::themed_text::ThemedText;
use crate::components::text_input::TextInput;
use crate::hooks::use_exit::use_exit_on_ctrl_cd_with_keybindings;
use crate::types::permissions::{PermissionBehavior, PermissionRuleValue};
use crate::utils::permissions::permission_rule_parser::{
    permission_rule_value_from_string, permission_rule_value_to_string,
};
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionRuleInputSubmit {
    pub rule_value: PermissionRuleValue,
    pub rule_behavior: PermissionBehavior,
}

/// Maps to: CC `PermissionRuleInput.tsx#handleSubmit`.
pub fn permission_rule_input_submit(
    value: &str,
    rule_behavior: PermissionBehavior,
) -> Option<PermissionRuleInputSubmit> {
    let trimmed = value.trim_matches(|character| {
        matches!(character,
            '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' |
            '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
            '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    });
    if trimmed.is_empty() {
        return None;
    }
    Some(PermissionRuleInputSubmit {
        rule_value: permission_rule_value_from_string(trimmed),
        rule_behavior,
    })
}

fn behavior_label(behavior: PermissionBehavior) -> &'static str {
    match behavior {
        PermissionBehavior::Allow => "allow",
        PermissionBehavior::Deny => "deny",
        PermissionBehavior::Ask => "ask",
    }
}

#[derive(Default, Props)]
pub struct PermissionRuleInputProps<'a> {
    pub rule_behavior: Option<PermissionBehavior>,
    pub on_submit: HandlerMut<'a, PermissionRuleInputSubmit>,
    pub on_cancel: HandlerMut<'a, ()>,
}

/// Maps to: CC `PermissionRuleInput.tsx:30-106#PermissionRuleInput`.
#[component]
pub fn PermissionRuleInput<'a>(
    props: &mut PermissionRuleInputProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let behavior = props.rule_behavior.unwrap_or(PermissionBehavior::Allow);
    let input_value = hooks.use_state(String::new);
    let cursor_offset = hooks.use_state(|| 0usize);
    let mut pending_submit = hooks.use_state(|| None::<String>);
    let mut pending_cancel = hooks.use_state(|| false);
    let exit_state = use_exit_on_ctrl_cd_with_keybindings(&mut hooks, true);
    let runtime = hooks
        .try_use_context::<crate::keybindings::keybinding_context::KeybindingRuntime>()
        .map(|runtime| runtime.clone());
    crate::keybindings::use_keybinding::use_keybinding(
        &mut hooks,
        runtime,
        "confirm:no",
        crate::keybindings::types::ContextName::Settings,
        || true,
        move || {
            pending_cancel.set(true);
            true
        },
    );
    let (columns, _) = hooks.use_terminal_size();
    let submitted = pending_submit.read().clone();
    if let Some(value) = submitted {
        pending_submit.set(None);
        if let Some(value) = permission_rule_input_submit(&value, behavior) {
            (props.on_submit)(value);
        }
    }
    if pending_cancel.get() {
        pending_cancel.set(false);
        (props.on_cancel)(());
    }
    let mut web = StyledSegment::new(permission_rule_value_to_string(&PermissionRuleValue::new(
        crate::tools::web_fetch_tool::prompt::WEB_FETCH_TOOL_NAME,
        None,
    )));
    web.styles.bold = Some(true);
    let mut bash = StyledSegment::new(permission_rule_value_to_string(&PermissionRuleValue::new(
        crate::tools::bash_tool::tool_name::BASH_TOOL_NAME,
        Some("ls:*".to_string()),
    )));
    bash.styles.bold = Some(true);
    let example = vec![
        StyledSegment::new(
            "Permission rules are a tool name, optionally followed by a specifier in parentheses.\ne.g., ",
        ),
        web,
        StyledSegment::new(" or "),
        bash,
    ];
    element! {
        View(flex_direction: FlexDirection::Column) {
            View(flex_direction: FlexDirection::Column, gap: 1u32, border_style: BorderStyle::Round, padding_left: 1u32, padding_right: 1u32, border_color: theme.permission) {
                Text(content: format!("Add {} permission rule", behavior_label(behavior)), color: theme.permission, weight: Weight::Bold)
                View(flex_direction: FlexDirection::Column) {
                    Text(segments: Some(example), wrap: TextWrap::Wrap)
                    View(border_dim_color: true, border_style: BorderStyle::Round, margin_top: 1u32, margin_bottom: 1u32, padding_left: 1u32) {
                        // CC BaseTextInput handles Escape without stopping the later
                        // PermissionRuleInput Settings listener. The native input
                        // consumes editing events, so restore that cancel lane.
                        TextInput(value: Some(input_value), cursor_offset: Some(cursor_offset), show_cursor: true,
                            escape_event_passthrough: true,
                            columns: columns.saturating_sub(6) as usize,
                            placeholder: Some(format!("Enter permission rule{}", crate::constants::figures::figures().ellipsis)),
                            on_submit: move |value| pending_submit.set(Some(value)),
                        )
                    }
                }
            }
            View(margin_left: 3u32) {
                ThemedText(content: exit_state.key_name.map(|key| format!("Press {key} again to exit")).unwrap_or_else(|| "Enter to submit · Esc to cancel".to_string()), dim_color: true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[derive(Default, Props)]
    struct PermissionInputTestProvidersProps {
        children: Vec<AnyElement<'static>>,
    }

    struct PermissionInputTestProviders;

    impl Component for PermissionInputTestProviders {
        type Props<'a> = PermissionInputTestProvidersProps;

        fn new(_props: &Self::Props<'_>) -> Self {
            Self
        }

        fn update(
            &mut self,
            props: &mut Self::Props<'_>,
            mut hooks: Hooks,
            updater: &mut ComponentUpdater,
        ) {
            let runtime = crate::keybindings::keybinding_provider_setup::use_keybinding_setup(
                &mut hooks,
                crate::keybindings::keybinding_context::KeybindingRuntime::with_default_bindings(),
            );
            // Match ContextProvider's retained move-only children: local runtime
            // updates borrow the same subtree instead of draining its props.
            let mut context = Context::owned(runtime);
            updater.set_transparent_layout(true);
            updater.update_children(props.children.iter_mut(), Some(context.borrow()));
        }
    }

    fn input_test_providers(child: AnyElement<'static>) -> AnyElement<'static> {
        crate::utils::process_runtime::initialize_test_process_runtime();
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                PermissionInputTestProviders {
                    ContextProvider(value: Context::owned(crate::state::store::AppStore::new(Default::default(), None))) {
                        FocusScope(handle_keys: false) { #(Some(child)) }
                    }
                }
            }
        }.into_any()
    }

    #[test]
    fn permission_rule_input_submit_parses_official_rule_strings() {
        let submitted = permission_rule_input_submit("  Bash(ls:*) ", PermissionBehavior::Allow)
            .expect("non-empty submit");
        assert_eq!(submitted.rule_behavior, PermissionBehavior::Allow);
        assert_eq!(submitted.rule_value.tool_name, "Bash");
        assert_eq!(submitted.rule_value.rule_content.as_deref(), Some("ls:*"));
        assert_eq!(
            permission_rule_input_submit("   ", PermissionBehavior::Deny),
            None
        );
    }

    #[test]
    fn permission_rule_input_renders_official_copy_and_examples() {
        let text = input_test_providers(
            element! {
                PermissionRuleInput(rule_behavior: Some(PermissionBehavior::Ask))
            }
            .into_any(),
        )
        .render(Some(100))
        .to_string();
        assert!(text.contains("Add ask permission rule"), "canvas=\n{text}");
        assert!(
            text.contains("Permission rules are a tool name"),
            "canvas=\n{text}"
        );
        assert!(text.contains("WebFetch"), "canvas=\n{text}");
        assert!(text.contains("Bash(ls:*)"), "canvas=\n{text}");
    }

    #[test]
    fn permission_rule_input_trim_matches_official_ecmascript_boundary() {
        assert!(permission_rule_input_submit("\u{feff} ", PermissionBehavior::Allow).is_none());
        assert_eq!(
            permission_rule_input_submit("\u{85}", PermissionBehavior::Ask)
                .unwrap()
                .rule_value
                .tool_name,
            "\u{85}"
        );
        assert_eq!(
            permission_rule_input_submit("\u{feff}Bash(ls:*)\u{feff}", PermissionBehavior::Deny)
                .unwrap()
                .rule_value
                .rule_content
                .as_deref(),
            Some("ls:*")
        );
    }

    #[tokio::test]
    async fn permission_rule_input_matches_official_empty_typing_submit_and_escape() {
        use futures::{StreamExt, stream};
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        let submitted = Arc::new(Mutex::new(Vec::new()));
        let cancelled = Arc::new(Mutex::new(0));
        let outputs = submitted.clone();
        let cancellations = cancelled.clone();
        let mut app = input_test_providers(
            element! {
                PermissionRuleInput(rule_behavior: Some(PermissionBehavior::Ask),
                    on_submit: move |value| outputs.lock().unwrap().push(value),
                    on_cancel: move |_| *cancellations.lock().unwrap() += 1,
                )
            }
            .into_any(),
        );
        let key = |code| TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, code));
        let events = stream::iter(vec![
            key(KeyCode::Enter),
            key(KeyCode::Char('n')),
            key(KeyCode::Backspace),
            TerminalEvent::Paste("  Bash(ls:*)  ".to_string()),
            key(KeyCode::Enter),
            key(KeyCode::Esc),
        ])
        .then(|event| async move {
            futures_timer::Delay::new(Duration::from_millis(20)).await;
            event
        })
        .chain(stream::pending());
        let mut frames =
            Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(100, 18),
            ));
        let deadline = futures_timer::Delay::new(Duration::from_secs(1));
        tokio::pin!(deadline);
        loop {
            tokio::select! {
                _ = &mut deadline => break,
                frame = frames.next() => { if frame.is_none() || *cancelled.lock().unwrap() != 0 { break; } }
            }
        }
        assert_eq!(*cancelled.lock().unwrap(), 1);
        assert_eq!(
            *submitted.lock().unwrap(),
            vec![PermissionRuleInputSubmit {
                rule_value: PermissionRuleValue::new("Bash", Some("ls:*".to_string())),
                rule_behavior: PermissionBehavior::Ask
            }]
        );
    }

    #[test]
    fn permission_rule_input_matches_official_inactive_footer_and_unfocused_placeholder() {
        let canvas =
            input_test_providers(element! { PermissionRuleInput }.into_any()).render(Some(100));
        let text = canvas.to_string();
        for (needle, placeholder) in [
            ("Enter permission rule", true),
            ("Enter to submit · Esc to cancel", false),
        ] {
            let (y, line) = text
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains(needle))
                .expect("source copy");
            let x = line[..line.find(needle).unwrap()].chars().count();
            let style = canvas.resolved_text_style(x, y).unwrap();
            assert!(!style.invert, "{needle}");
            assert_eq!(style.dim, placeholder, "{needle}");
            if !placeholder {
                assert_eq!(style.color, Some(theme::current().inactive));
            }
        }
    }

    #[tokio::test]
    async fn permission_rule_input_matches_official_pending_exit_footer_color() {
        use futures::StreamExt;
        let (keys, events) = async_channel::unbounded();
        let mut app = input_test_providers(element! { PermissionRuleInput }.into_any());
        let mut frames =
            Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(100, 18),
            ));
        let mut last = String::new();
        let exercise = async {
            let mut sent = false;
            while let Some(canvas) = frames.next().await {
                last = canvas.to_string();
                if !sent && last.contains("Enter to submit · Esc to cancel") {
                    let mut key = KeyEvent::new(KeyEventKind::Press, KeyCode::Char('c'));
                    key.modifiers = KeyModifiers::CONTROL;
                    keys.send(TerminalEvent::Key(key)).await.unwrap();
                    sent = true;
                } else if sent {
                    let needle = "Press Ctrl-C again to exit";
                    if let Some((y, line)) = last
                        .lines()
                        .enumerate()
                        .find(|(_, line)| line.contains(needle))
                    {
                        let x = line[..line.find(needle).unwrap()].chars().count();
                        let style = canvas.resolved_text_style(x, y).unwrap();
                        assert_eq!(style.color, Some(theme::current().inactive));
                        assert!(!style.dim);
                        return true;
                    }
                }
            }
            false
        };
        let done = crate::utils::race(exercise, async {
            futures_timer::Delay::new(std::time::Duration::from_secs(3)).await;
            false
        })
        .await;
        assert!(done, "source pending exit footer: {last}");
    }
}
