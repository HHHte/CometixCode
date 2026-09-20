//! Maps to: CC `src/screens/` — application screens/use-case owners.
//! `repl.rs` mirrors official `screens/REPL.tsx`: it owns conversation state,
//! current local screen, prompt submission wiring, and command-local UI routing.

pub mod doctor;
pub mod repl;
pub mod resume_conversation;
