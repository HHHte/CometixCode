//! Maps to: CC `utils/teleport/environments.ts`.
//!
//! Network helpers (`fetchEnvironments`, `createDefaultCloudEnvironment`) are
//! intentionally not implemented in this UI slice. They require Claude.ai OAuth
//! and organization context. The Rust boundary keeps the official response and
//! resource shapes for pure UI/state selection.

/// Maps to: CC `utils/teleport/environments.ts` `EnvironmentKind`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentKind {
    AnthropicCloud,
    Byoc,
    Bridge,
}

impl EnvironmentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AnthropicCloud => "anthropic_cloud",
            Self::Byoc => "byoc",
            Self::Bridge => "bridge",
        }
    }
}

/// Maps to: CC `utils/teleport/environments.ts` `EnvironmentState`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentState {
    Active,
}

impl EnvironmentState {
    pub fn as_str(&self) -> &'static str {
        "active"
    }
}

/// Maps to: CC `utils/teleport/environments.ts` `EnvironmentResource`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentResource {
    pub kind: EnvironmentKind,
    pub environment_id: String,
    pub name: String,
    pub created_at: String,
    pub state: EnvironmentState,
}

impl EnvironmentResource {
    pub fn new(
        kind: EnvironmentKind,
        environment_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            environment_id: environment_id.into(),
            name: name.into(),
            created_at: String::new(),
            state: EnvironmentState::Active,
        }
    }
}

/// Maps to: CC `utils/teleport/environments.ts` `EnvironmentListResponse`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentListResponse {
    pub environments: Vec<EnvironmentResource>,
    pub has_more: bool,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_resource_shape_matches_official_string_values() {
        let env = EnvironmentResource::new(EnvironmentKind::AnthropicCloud, "env_1", "Cloud");
        assert_eq!(env.kind.as_str(), "anthropic_cloud");
        assert_eq!(EnvironmentKind::Byoc.as_str(), "byoc");
        assert_eq!(EnvironmentKind::Bridge.as_str(), "bridge");
        assert_eq!(env.state.as_str(), "active");
        assert_eq!(env.environment_id, "env_1");
        assert_eq!(env.name, "Cloud");
    }
}
