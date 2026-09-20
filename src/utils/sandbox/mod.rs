//! Maps to: CC `utils/sandbox/`.
//!
//! Settings/UI helpers live in [`sandbox_adapter`]. Native runtime dependencies
//! used by Bash (domain-filtering proxies and Linux bridges) live in
//! [`network_proxy`]; the shared violation store remains on the adapter.

pub mod network_proxy;
pub mod sandbox_adapter;
