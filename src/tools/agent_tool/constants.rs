//! Maps to CC `tools/AgentTool/constants.ts`.

pub const AGENT_TOOL_NAME: &str = "Agent";
pub const LEGACY_AGENT_TOOL_NAME: &str = "Task";
pub const VERIFICATION_AGENT_TYPE: &str = "verification";

/// Maps to CC `tools/AgentTool/constants.ts#ONE_SHOT_BUILTIN_AGENT_TYPES`.
pub const ONE_SHOT_BUILTIN_AGENT_TYPES: &[&str] = &["Explore", "Plan"];

/// Maps to CC `ONE_SHOT_BUILTIN_AGENT_TYPES.has(agentType)`.
pub fn is_one_shot_builtin_agent_type(agent_type: &str) -> bool {
    ONE_SHOT_BUILTIN_AGENT_TYPES.contains(&agent_type)
}
