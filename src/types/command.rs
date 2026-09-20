//! Maps to: CC `types/command.ts`.

/// Maps to: CC `types/command.ts::ResumeEntrypoint`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumeEntrypoint {
    CliFlag,
    SlashCommandPicker,
    SlashCommandSessionId,
    SlashCommandTitle,
    Fork,
}
