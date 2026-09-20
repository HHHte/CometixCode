//! Deterministic agent ID helpers for swarm/teammate routing.
//!
//! Maps to: CC `utils/agentId.ts`.

/// Maps to: CC `utils/agentId.ts#formatAgentId`.
pub fn format_agent_id(agent_name: &str, team_name: &str) -> String {
    format!("{agent_name}@{team_name}")
}

/// Maps to: CC `utils/agentId.ts#parseAgentId`.
pub fn parse_agent_id(agent_id: &str) -> Option<(String, String)> {
    let at_index = agent_id.find('@')?;
    Some((
        agent_id[..at_index].to_string(),
        agent_id[at_index + 1..].to_string(),
    ))
}

/// Maps to: CC `utils/agentId.ts#generateRequestId`.
pub fn generate_request_id(request_type: &str, agent_id: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp_millis().max(0);
    format!("{request_type}-{timestamp}@{agent_id}")
}

/// Maps to: CC `utils/agentId.ts#parseRequestId`.
pub fn parse_request_id(request_id: &str) -> Option<(String, u64, String)> {
    let at_index = request_id.find('@')?;
    let prefix = &request_id[..at_index];
    let agent_id = &request_id[at_index + 1..];
    let dash_index = prefix.rfind('-')?;
    let request_type = &prefix[..dash_index];
    let timestamp = prefix[dash_index + 1..].parse::<u64>().ok()?;
    Some((request_type.to_string(), timestamp, agent_id.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_and_parses_agent_ids_like_official_separator_contract() {
        let id = super::format_agent_id("researcher", "my-team");
        assert_eq!(id, "researcher@my-team");
        assert_eq!(
            super::parse_agent_id(&id),
            Some(("researcher".to_string(), "my-team".to_string()))
        );
        assert_eq!(super::parse_agent_id("researcher"), None);
    }

    #[test]
    fn parses_request_ids_by_first_at_and_last_dash_like_official() {
        let request = "shutdown-1702500000000@researcher@my-project";
        assert_eq!(
            super::parse_request_id(request),
            Some((
                "shutdown".to_string(),
                1_702_500_000_000,
                "researcher@my-project".to_string()
            ))
        );
        assert!(super::generate_request_id("plan", "agent@team").starts_with("plan-"));
        assert_eq!(super::parse_request_id("shutdown@agent@team"), None);
        assert_eq!(super::parse_request_id("shutdown-not-a-number@agent"), None);
    }
}
