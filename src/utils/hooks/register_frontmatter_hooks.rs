//! Frontmatter hook registration.
//!
//! Maps to: CC `utils/hooks/registerFrontmatterHooks.ts`.

use super::session_hooks::add_session_hook;
use crate::services::hooks::{HOOK_EVENTS, HookEvent, HooksConfig};

/// Register hooks from agent/skill frontmatter into session-scoped hooks.
///
/// Maps to: CC `registerFrontmatterHooks(...)`. When `is_agent` is true,
/// `Stop` hooks are converted to `SubagentStop`, matching the official agent
/// lifecycle where subagents fire `SubagentStop` on completion.
pub fn register_frontmatter_hooks(
    session_id: &str,
    hooks: &HooksConfig,
    _source_name: &str,
    is_agent: bool,
) -> usize {
    if hooks.is_empty() {
        return 0;
    }

    let mut hook_count = 0usize;
    for event in HOOK_EVENTS {
        let Some(matchers) = hooks.get(event.as_str()) else {
            continue;
        };
        if matchers.is_empty() {
            continue;
        }

        let target_event = if is_agent && *event == HookEvent::Stop {
            HookEvent::SubagentStop
        } else {
            *event
        };

        for matcher_config in matchers {
            let matcher = matcher_config.matcher.clone().unwrap_or_default();
            if matcher_config.hooks.is_empty() {
                continue;
            }
            for hook in &matcher_config.hooks {
                add_session_hook(session_id, target_event, matcher.clone(), hook.clone());
                hook_count += 1;
            }
        }
    }

    hook_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::hooks::{HookCommand, HookConfigEntry};
    use crate::utils::hooks::session_hooks;

    fn hook(command: &str) -> HookCommand {
        HookCommand {
            command: command.to_string(),
            shell: None,
            timeout: Some(5),
            condition: None,
            status: None,
            once: None,
            is_async: None,
            async_rewake: None,
        }
    }

    #[test]
    fn register_frontmatter_hooks_converts_agent_stop_to_subagent_stop() {
        session_hooks::clear_all_session_hooks();
        let mut config = HooksConfig::new();
        config.insert(
            "Stop".to_string(),
            vec![HookConfigEntry {
                matcher: None,
                hooks: vec![hook("echo stop")],
                plugin_root: None,
                plugin_name: None,
                plugin_id: None,
            }],
        );

        let count = register_frontmatter_hooks("agent-1", &config, "agent 'reviewer'", true);
        assert_eq!(count, 1);
        assert!(session_hooks::get_session_hooks("agent-1", Some(HookEvent::Stop)).is_empty());
        let subagent_stop =
            session_hooks::get_session_hooks("agent-1", Some(HookEvent::SubagentStop));
        let entries = subagent_stop.get("SubagentStop").expect("subagent stop");
        assert_eq!(entries[0].hooks[0].command, "echo stop");
        session_hooks::clear_all_session_hooks();
    }
}
