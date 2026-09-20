//! Maps to CC `commands/login/login.tsx`.

use crate::commands::Command;
use crate::components::console_oauth_flow::{ConsoleOAuthFlow, ConsoleOAuthMode, OAuthStatus};
use crate::components::design_system::dialog::Dialog;
use crate::tool::ToolUseContext;
use crate::utils::process_user_input::ProcessUserInputBaseResult;
use crate::utils::process_user_input::process_slash_command::{
    LocalCommandUi, SlashCommandAction, SlashCommandInvocation,
};
use crate::utils::theme::Theme;
use iocraft::prelude::*;

/// Source-shaped local JSX dispatch. `ConsoleOAuthFlow` remains the render
/// owner; REPL owns mounting and completion exactly like other local-jsx UIs.
pub fn dispatch(
    command: &Command,
    args: &str,
    _uuid: Option<String>,
    _context: &ToolUseContext,
) -> ProcessUserInputBaseResult {
    ProcessUserInputBaseResult {
        messages: Vec::new(),
        should_query: false,
        allowed_tools: None,
        local_action: Some(SlashCommandAction::OpenLocalCommandUi {
            command: LocalCommandUi::Login {
                args: args.to_string(),
            },
            invocation: SlashCommandInvocation::new(command.name.as_ref(), args),
        }),
        query_source: crate::constants::query_source::QuerySource::Prompt,
    }
}

#[derive(Default, Props)]
pub struct LoginCommandProps<'a> {
    pub on_done: HandlerMut<'a, String>,
}

/// Render-only login flow backed by the existing ConsoleOAuthFlow. The final
/// OAuth browser/token side effects remain behind the explicit product gate.
#[component]
pub fn LoginCommand<'a>(
    props: &mut LoginCommandProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let mut pending = hooks.use_state(|| Option::<String>::None);
    hooks.use_terminal_events({
        let mut pending = pending;
        move |event| {
            let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
                return;
            };
            if kind == KeyEventKind::Release {
                return;
            }
            match code {
                KeyCode::Enter => pending.set(Some(
                    "OAuth login is unavailable in this build (credential side effects are disabled)."
                        .to_string(),
                )),
                KeyCode::Esc => pending.set(Some("Login interrupted".to_string())),
                _ => {}
            }
        }
    });
    let output = pending.read().clone();
    if let Some(output) = output {
        pending.set(None);
        (props.on_done)(output);
    }
    element! {
        Dialog(
            title: "Login".to_string(),
            color: Some(theme.permission),
            on_cancel: move |_| pending.set(Some("Login interrupted".to_string())),
        ) {
            ConsoleOAuthFlow(
                oauth_status: OAuthStatus::Idle,
                mode: ConsoleOAuthMode::Login,
            )
            Text(
                content: "OAuth browser and credential writes are disabled in this build.".to_string(),
                color: theme.inactive,
                wrap: TextWrap::Wrap,
            )
        }
    }
}
