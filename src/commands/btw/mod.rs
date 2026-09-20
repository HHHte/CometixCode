//! Maps to: CC `commands/btw/index.ts`.

pub mod btw;

pub const NAME: &str = "btw";
pub const DESCRIPTION: &str =
    "Ask a quick side question without interrupting the main conversation";

pub use btw::{BtwSideQuestion, call};
