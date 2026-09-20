//! Team-memory synchronization safety owners.
//!
//! Maps to: CC `services/teamMemorySync/*`.

#[cfg(feature = "anthropic_internal")]
pub mod secret_scanner;
pub mod team_mem_secret_guard;
