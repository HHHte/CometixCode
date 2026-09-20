//! Built-in agent registry.
//!
//! Maps to CC `tools/AgentTool/builtInAgents.ts`.
//!
//! Individual built-in agent definitions live under `built_in/*`, matching CC
//! `tools/AgentTool/built-in/*`. This module only performs the official
//! registry/gating assembly.

use super::built_in::{
    claude_code_guide_agent::claude_code_guide_agent, explore_agent::explore_agent,
    general_purpose_agent::general_purpose_agent, plan_agent::plan_agent,
    statusline_setup::statusline_setup_agent, verification_agent::verification_agent,
};
use super::load_agents_dir::AgentDefinition;

/// Maps to CC `areExplorePlanAgentsEnabled()`.
///
/// Maps to: CC `builtInAgents.ts:13-20` `areExplorePlanAgentsEnabled` —
/// `feature('BUILTIN_EXPLORE_PLAN_AGENTS')` is ON in production builds
/// (`scripts/build.ts:44`), then the `tengu_amber_stoat` GrowthBook gate
/// (fallback true). GrowthBook delivery is out of scope for this port
/// (user ruling): the gate lives as a hardcoded switch-table entry.
pub fn are_explore_plan_agents_enabled_readonly() -> bool {
    crate::utils::feature_flags::feature_enabled(
        crate::utils::feature_flags::FeatureFlag::BuiltinExplorePlanAgents,
    )
}

/// Maps to CC `getBuiltInAgents()`.
pub fn get_built_in_agents_readonly(
    get_env: &impl Fn(&str) -> Option<String>,
) -> Vec<AgentDefinition> {
    get_built_in_agents_from_snapshot(
        get_env,
        crate::bootstrap::state::get_is_non_interactive_session(),
    )
}

fn get_built_in_agents_from_snapshot(
    get_env: &impl Fn(&str) -> Option<String>,
    is_non_interactive_session: bool,
) -> Vec<AgentDefinition> {
    // Maps to official SDK/non-interactive escape hatch.
    if crate::utils::env_utils::is_env_truthy(
        get_env("CLAUDE_AGENT_SDK_DISABLE_BUILTIN_AGENTS").as_deref(),
    ) && is_non_interactive_session
    {
        return Vec::new();
    }

    let mut agents = vec![general_purpose_agent(), statusline_setup_agent()];

    if are_explore_plan_agents_enabled_readonly() {
        agents.extend([explore_agent(), plan_agent()]);
    }

    // Maps to official non-SDK entrypoint check.
    if !matches!(
        get_env("CLAUDE_CODE_ENTRYPOINT").as_deref(),
        Some("sdk-ts" | "sdk-py" | "sdk-cli")
    ) {
        agents.push(claude_code_guide_agent());
    }

    if is_verification_agent_enabled_readonly() {
        agents.push(verification_agent());
    }

    // The coordinator built-ins are feature/GrowthBook gated upstream and are
    // not materialized until those feature snapshots exist.
    agents
}

/// Maps to CC `builtInAgents.ts` verification feature/GrowthBook gate.
pub fn is_verification_agent_enabled_readonly() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::PermissionMode;

    #[test]
    fn built_in_agents_match_official_default_and_sdk_entrypoint_gate() {
        // CC builtInAgents.ts:45-52: base pair, then Explore/Plan (gate ON in
        // production external builds — build.ts:44 + tengu_amber_stoat
        // fallback true), then guide for non-SDK entrypoints.
        let default = get_built_in_agents_from_snapshot(&|_| None, false);
        assert_eq!(
            default
                .iter()
                .map(|agent| agent.agent_type.as_str())
                .collect::<Vec<_>>(),
            vec![
                "general-purpose",
                "statusline-setup",
                "Explore",
                "Plan",
                "claude-code-guide"
            ]
        );

        // The SDK entrypoint drops only the guide (:55-62); Explore/Plan stay.
        let sdk = get_built_in_agents_from_snapshot(
            &|key| (key == "CLAUDE_CODE_ENTRYPOINT").then(|| "sdk-ts".to_string()),
            false,
        );
        assert_eq!(
            sdk.iter()
                .map(|agent| agent.agent_type.as_str())
                .collect::<Vec<_>>(),
            vec!["general-purpose", "statusline-setup", "Explore", "Plan"]
        );
    }

    #[test]
    fn built_in_agents_honor_noninteractive_sdk_disable_gate() {
        let disabled = get_built_in_agents_from_snapshot(
            &|key| (key == "CLAUDE_AGENT_SDK_DISABLE_BUILTIN_AGENTS").then(|| "true".to_string()),
            true,
        );
        assert!(disabled.is_empty());

        let interactive = get_built_in_agents_from_snapshot(
            &|key| (key == "CLAUDE_AGENT_SDK_DISABLE_BUILTIN_AGENTS").then(|| "true".to_string()),
            false,
        );
        assert!(!interactive.is_empty());
    }

    #[test]
    fn built_in_agent_registry_assembles_official_agent_modules() {
        let agents = get_built_in_agents_from_snapshot(&|_| None, false);
        let statusline = agents
            .iter()
            .find(|agent| agent.agent_type == "statusline-setup")
            .unwrap();
        assert_eq!(statusline.model.as_deref(), Some("sonnet"));
        assert_eq!(
            statusline.tools.as_deref(),
            Some(&["Read".to_string(), "Edit".to_string()][..])
        );

        let guide = agents
            .iter()
            .find(|agent| agent.agent_type == "claude-code-guide")
            .unwrap();
        assert_eq!(guide.model.as_deref(), Some("haiku"));
        assert_eq!(guide.permission_mode, Some(PermissionMode::DontAsk));
    }
}
