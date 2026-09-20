//! Maps to: CC `commands/rename/index.ts`, `rename.ts`, and
//! `generateSessionName.ts`.

pub mod generate_session_name;
pub mod rename;

pub use rename::{RenameCall, RenameGenerationRequest};
