//! Maps to: CC `commands/output-style/output-style.tsx`.
/// Maps to CC `call`: synchronous carrier for immediately completed onDone.
pub fn call(
    command: &crate::commands::Command,
    args: &str,
    uuid: Option<String>,
    _context: &crate::tool::ToolUseContext,
) -> crate::utils::process_user_input::ProcessUserInputBaseResult {
    crate::utils::process_user_input::process_slash_command::system_display_local_command_result(
        uuid,
        command.name.as_ref(),
        args,
        "/output-style has been deprecated. Use /config to change your output style, or set it in your settings file. Changes take effect on the next session.",
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn output_style_matches_official_system_only_deprecation() {
        // CC output-style.tsx:4-8: exactly one onDone, display system, no write.
        let command = super::super::command();
        let result = command.call.unwrap()(&command, "explanatory", None, &Default::default());
        assert!(!result.should_query);
        assert!(result.local_action.is_none());
        assert_eq!(result.messages.len(), 2);
        assert!(result.messages.iter().all(|message| matches!(
            message.kind,
            crate::types::message::RenderableMessageKind::System(_)
        )));
    }
}
