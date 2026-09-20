//! Built-in plugin initialization.
//!
//! Maps to: CC `plugins/bundled/index.ts`.

/// Initialize built-in plugins that ship with the CLI.
///
/// Maps to: CC `plugins/bundled/index.ts#initBuiltinPlugins`.
///
/// Official source currently registers no built-in plugins; this no-op keeps
/// the startup boundary in place so future built-in registrations belong here
/// instead of inside plugin loading, settings, or UI code.
pub fn init_builtin_plugins() {
    // No built-in plugins registered yet — this mirrors the official scaffold.
}
