//! Maps to: CC `entrypoints/`.
//!
//! Process-level entry surfaces. Interactive TUI launch continues through
//! [`cli`] → [`crate::main`] (`main.tsx`); setup gates live under
//! `interactive_helpers` (CC `showSetupScreens`).

pub mod cli;
pub mod sandbox_types;
