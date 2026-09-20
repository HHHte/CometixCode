//! Maps to CC `commands/ide/index.ts` and `commands/ide/ide.tsx`.

pub mod ide;

/// Maps to CC `commands/ide/index.ts`.
pub fn command() -> crate::commands::Command {
    crate::commands::Command::local_ui("ide", "Manage IDE integrations and show status")
        .argument_hint("[open]")
        .executable(crate::commands::ide::ide::dispatch)
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptor_matches_official_metadata() {
        let command = super::command();
        assert_eq!(command.name, "ide");
        assert_eq!(command.argument_hint.as_deref(), Some("[open]"));
        assert!(command.call.is_some());
    }
}
