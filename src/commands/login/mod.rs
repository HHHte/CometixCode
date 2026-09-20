//! Maps to CC `commands/login/index.ts` and `commands/login/login.tsx`.
//!
//! The command remains a local JSX boundary. The OAuth transport is deliberately
//! behind the product's existing `OAUTH_CREDENTIAL_SIDE_EFFECTS_ENABLED` gate;
//! this module owns the command descriptor and the source-shaped login panel.

pub mod login;

/// Maps to CC `commands/login/index.ts`.
pub fn command() -> crate::commands::Command {
    crate::commands::Command::local_ui("login", "Sign in with your Anthropic account")
        .description_when(|| {
            if crate::utils::auth::has_anthropic_api_key_auth() {
                "Switch Anthropic accounts".to_string()
            } else {
                "Sign in with your Anthropic account".to_string()
            }
        })
        .executable(crate::commands::login::login::dispatch)
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptor_is_local_ui_and_has_live_description_owner() {
        let command = super::command();
        assert_eq!(command.name, "login");
        assert_eq!(command.kind, crate::commands::CommandKind::LocalUi);
        assert!(command.call.is_some());
    }
}
