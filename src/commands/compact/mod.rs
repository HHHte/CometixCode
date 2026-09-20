//! `/compact` command registration boundary.
//!
//! Maps to CC `commands/compact/index.ts`.

pub mod compact;

pub const NAME: &str = "compact";
pub const DESCRIPTION: &str = "Clear conversation history but keep a summary in context. Optional: /compact [instructions for summarization]";
