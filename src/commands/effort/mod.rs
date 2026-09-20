//! Maps to: CC `commands/effort/index.ts` (2.1.241 `qhE` / `Krh`).
//!
//! Command descriptor only. Implementation lives in [`effort`]. Aggregate
//! registration remains in `commands/mod.rs`, matching CC `commands.ts`.

use crate::utils::ultracode::{get_eligible_effort_levels, is_ultracode_available};

pub mod effort;

pub const NAME: &str = "effort";
pub const DESCRIPTION: &str = "Set effort level for model usage";

/// Maps to: CC `Krh` — `qhE.get argumentHint()` with brackets `[` `]`.
pub fn effort_argument_hint(model: &str) -> String {
    let ultracode = if is_ultracode_available(Some(model)) {
        "|ultracode"
    } else {
        ""
    };
    format!(
        "[{}{ultracode}|auto]",
        get_eligible_effort_levels(model).join("|")
    )
}

/// Maps to: CC `qhE.argumentHint` → `Krh("[", "]")` with `$i()`.
pub fn current_effort_argument_hint() -> String {
    effort_argument_hint(&crate::utils::model::model::get_session_main_loop_model())
}
