//! Maps to: CC `utils/hooks/` auxiliary modules.
//! The `utils/hooks.ts` runtime remains in `crate::services::hooks`.

pub mod api_query_hook_helper;
pub mod async_hook_registry;
pub mod exec_agent_hook;
pub mod exec_http_hook;
pub mod exec_prompt_hook;
pub mod file_changed_watcher;
pub mod hook_events;
pub mod hook_helpers;
pub mod hooks_config_manager;
pub mod hooks_config_snapshot;
pub mod hooks_settings;
pub mod post_sampling_hooks;
pub mod register_frontmatter_hooks;
pub mod register_skill_hooks;
pub mod session_hooks;
pub mod skill_improvement;
pub mod ssrf_guard;
