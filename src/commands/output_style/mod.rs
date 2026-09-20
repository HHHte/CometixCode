//! Maps to: CC `commands/output-style/index.ts`.
pub mod output_style;
pub fn command() -> super::Command {
    super::Command::local_ui(
        "output-style",
        "Deprecated: use /config to change output style",
    )
    .hidden()
    .executable(output_style::call)
}
