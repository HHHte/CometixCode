//! Maps to: CC `commands/advisor.ts`.

pub mod advisor;

/// Descriptor and deferred local-call transport for `/advisor`.
pub fn command() -> crate::commands::Command {
    crate::commands::Command::local("advisor", "Configure the advisor model")
        .argument_hint("[<model>|off]")
        .enabled_when(crate::utils::advisor::can_user_configure_advisor)
        .hidden_when(|| !crate::utils::advisor::can_user_configure_advisor())
        .supports_non_interactive()
        .executable(advisor::dispatch)
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptor_matches_official_local_advisor_command() {
        let command = super::command();
        assert_eq!(command.name, "advisor");
        assert_eq!(command.argument_hint.as_deref(), Some("[<model>|off]"));
        assert!(command.supports_non_interactive);
        assert!(command.call.is_some());
    }
}
