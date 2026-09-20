//! Backend type definitions.
//! Maps to: CC `utils/swarm/backends/types.ts`.

/// Maps to: CC `BackendType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendType {
    Tmux,
    ITerm2,
    InProcess,
}

impl BackendType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tmux => "tmux",
            Self::ITerm2 => "iterm2",
            Self::InProcess => "in-process",
        }
    }
}

/// Maps to: CC `PaneBackendType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneBackendType {
    Tmux,
    ITerm2,
}

impl PaneBackendType {
    pub fn as_backend_type(self) -> BackendType {
        match self {
            Self::Tmux => BackendType::Tmux,
            Self::ITerm2 => BackendType::ITerm2,
        }
    }
}

/// Maps to: CC `BackendDetectionResult` metadata, without a concrete backend
/// object until pane backends are fully ported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendDetectionResult {
    pub backend_type: PaneBackendType,
    pub is_native: bool,
    pub needs_it2_setup: bool,
}

/// Maps to: CC `types.ts#TeammateSpawnConfig`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeammateSpawnConfig {
    pub name: String,
    pub team_name: String,
    pub prompt: String,
    pub cwd: String,
    pub color: Option<crate::tools::agent_tool::agent_color_manager::AgentColorName>,
    pub plan_mode_required: bool,
    pub model: Option<String>,
    pub parent_session_id: String,
    /// Rust extension matching `spawnMultiAgent.ts` teammate identity args.
    pub agent_type: Option<String>,
}

/// Maps to: CC `types.ts#TeammateSpawnResult`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeammateSpawnResult {
    pub success: bool,
    pub agent_id: String,
    pub error: Option<String>,
    pub task_id: Option<String>,
    pub pane_id: Option<String>,
}

/// Maps to: CC `types.ts#TeammateMessage`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeammateMessage {
    pub text: String,
    pub from: String,
    pub color: Option<String>,
    pub timestamp: Option<String>,
    pub summary: Option<String>,
}

/// Maps to: CC `isPaneBackend(...)`.
pub fn is_pane_backend(backend_type: BackendType) -> bool {
    matches!(backend_type, BackendType::Tmux | BackendType::ITerm2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_backend_type_guard_matches_official() {
        assert!(is_pane_backend(BackendType::Tmux));
        assert!(is_pane_backend(BackendType::ITerm2));
        assert!(!is_pane_backend(BackendType::InProcess));
        assert_eq!(PaneBackendType::Tmux.as_backend_type().as_str(), "tmux");
    }
}
