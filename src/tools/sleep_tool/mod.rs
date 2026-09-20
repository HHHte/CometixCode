//! Sleep tool metadata.
//!
//! Maps to CC `tools/SleepTool/`. Only `prompt.ts` survives in the reference
//! build: `tools.ts:25-28` reaches `SleepTool.js` through
//! `feature('PROACTIVE') || feature('KAIROS')`, both off, so the tool module
//! itself is eliminated and its schema, `call()`, and `interruptBehavior` have
//! no source to port from. The surviving copy has exactly three live consumers
//! outside that gate, all ported:
//! - the REPL spinner gate `only_sleep_tool_active` (screens/repl.rs ↔
//!   REPL.tsx:2219-2231),
//! - the auto-mode classifier allowlist (utils/permissions/
//!   classifier_decision.rs ↔ classifierDecision.ts:87),
//! - the mid-turn queue-drain widening `sleepRan` (query.rs
//!   `drain_queued_commands_snapshot` ↔ query.ts:1566-1571).
//!
//! TODO(parity): register the tool in `get_all_base_tools()` once the proactive
//! runtime (`isProactiveActive()`, tick scheduling, queue-drain wake-up) lands
//! and an authoritative input schema is available.

pub mod prompt;
