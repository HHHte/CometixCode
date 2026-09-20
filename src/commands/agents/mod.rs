//! Maps to: CC `commands/agents/index.ts`.

pub mod agents;

pub use agents::AgentsCommand;

pub const NAME: &str = "agents";
pub const DESCRIPTION: &str = "Manage agent configurations";
