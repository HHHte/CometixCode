//! Maps to CC `commands/tasks/index.ts:3-11`.
pub mod tasks;
pub const NAME: &str = "tasks";
pub const DESCRIPTION: &str = "List and manage background tasks";
pub const ALIASES: &[&str] = &["bashes"];
pub use tasks::call;
