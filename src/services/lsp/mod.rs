//! LSP service layer.
//!
//! Maps to: CC `services/lsp/*`.
//!
//! This module is being ported in slices. Diagnostic registry, passive
//! diagnostic attachment shaping, plugin LSP discovery, manager initialization
//! state, extension routing, and stdio JSON-RPC client transport are available
//! now. Remaining work stays under the official LSPTool/service boundaries.

pub mod client;
pub mod config;
pub mod diagnostic_registry;
pub mod manager;
pub mod passive_feedback;
pub mod server_instance;
pub mod server_manager;
pub mod types;
