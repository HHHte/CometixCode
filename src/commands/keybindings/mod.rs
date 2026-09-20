//! Maps to: CC `commands/keybindings/index.ts`.

pub mod keybindings;

pub const NAME: &str = "keybindings";
pub const DESCRIPTION: &str = "Open or create your keybindings configuration file";

pub use keybindings::call;
