//! Prompt category helpers.
//! Maps to CC `utils/promptCategory.ts`.

use crate::constants::query_source::QuerySource;

/// Maps to CC `utils/promptCategory.ts:16-28` `getQuerySourceForAgent(...)`:
///
/// ```ts
/// if (isBuiltInAgent) {
///   // TODO: avoid this cast
///   return agentType
///     ? (`agent:builtin:${agentType}` as QuerySource)
///     : 'agent:default'
/// } else {
///   return 'agent:custom'
/// }
/// ```
///
/// The inner test is JS truthiness on a `string | undefined`, so an EMPTY
/// agent type is falsy and lands on `agent:default` alongside `undefined` —
/// hence `Option<&str>` plus the emptiness filter rather than a bare
/// `Option::map`.
pub fn get_query_source_for_agent(
    agent_type: Option<&str>,
    is_built_in_agent: bool,
) -> QuerySource {
    if !is_built_in_agent {
        return QuerySource::AgentCustom;
    }
    match agent_type.filter(|value| !value.is_empty()) {
        Some(agent_type) => QuerySource::agent_builtin(agent_type),
        None => QuerySource::AgentDefault,
    }
}

/// Maps to CC `utils/promptCategory.ts` `getQuerySourceForREPL()` default.
pub fn get_query_source_for_repl(source: &QuerySource) -> &str {
    match source {
        QuerySource::Prompt => "repl_main_thread",
        _ => source.as_api_source(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `promptCategory.ts:20-27`, all three arms.
    #[test]
    fn query_source_for_agent_matches_official_three_arms() {
        assert_eq!(
            get_query_source_for_agent(Some("fork"), true),
            QuerySource::agent_builtin("fork")
        );
        assert_eq!(
            get_query_source_for_agent(Some("general-purpose"), true),
            QuerySource::agent_builtin("general-purpose")
        );
        assert_eq!(
            get_query_source_for_agent(None, true),
            QuerySource::AgentDefault
        );
        assert_eq!(
            get_query_source_for_agent(Some("code-reviewer"), false),
            QuerySource::AgentCustom
        );
    }

    /// JS truthiness: `'' ? a : b` picks `b`, so an empty built-in agent type
    /// is `agent:default`, NOT `agent:builtin:`.
    #[test]
    fn an_empty_built_in_agent_type_is_falsy_and_becomes_agent_default() {
        assert_eq!(
            get_query_source_for_agent(Some(""), true),
            QuerySource::AgentDefault
        );
        // A custom agent never consults agentType at all.
        assert_eq!(
            get_query_source_for_agent(Some(""), false),
            QuerySource::AgentCustom
        );
    }
}
