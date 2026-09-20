//! Maps to: CC `commands/help/index.ts`.

use crate::commands::Command;

pub const NAME: &str = "help";
pub const DESCRIPTION: &str = "Show help and available commands";

/// Rust registry projection of the default export in CC
/// `commands/help/index.ts`.
pub fn command() -> Command {
    Command::local_ui(NAME, DESCRIPTION)
        .executable(crate::utils::process_user_input::process_slash_command::call_help)
}
