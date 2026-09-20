//! Maps to CC `commands/logout/index.ts` and `commands/logout/logout.tsx`.

pub mod logout;

/// Maps to CC `commands/logout/index.ts`.
pub fn command() -> crate::commands::Command {
    crate::commands::Command::local_ui("logout", "Sign out from your Anthropic account")
        .executable(crate::commands::logout::logout::dispatch)
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptor_is_local_ui_and_executable() {
        let command = super::command();
        assert_eq!(command.name, "logout");
        assert_eq!(command.kind, crate::commands::CommandKind::LocalUi);
        assert!(command.call.is_some());
    }
}
