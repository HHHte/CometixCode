//! Teammate mode snapshot.
//! Maps to: CC `utils/swarm/backends/teammateModeSnapshot.ts`.

use std::sync::{LazyLock, Mutex};

/// Maps to: CC `TeammateMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeammateMode {
    Auto,
    Tmux,
    InProcess,
}

impl TeammateMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Tmux => "tmux",
            Self::InProcess => "in-process",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "tmux" => Some(Self::Tmux),
            "in-process" => Some(Self::InProcess),
            _ => None,
        }
    }
}

#[derive(Default)]
struct TeammateModeSnapshotState {
    initial_teammate_mode: Option<TeammateMode>,
    cli_teammate_mode_override: Option<TeammateMode>,
}

static TEAMMATE_MODE_SNAPSHOT: LazyLock<Mutex<TeammateModeSnapshotState>> =
    LazyLock::new(|| Mutex::new(TeammateModeSnapshotState::default()));

fn mode_from_config(config: &crate::utils::config::GlobalConfig) -> TeammateMode {
    config
        .teammate_mode
        .as_deref()
        .and_then(TeammateMode::parse)
        .unwrap_or(TeammateMode::Auto)
}

/// Maps to: CC `setCliTeammateModeOverride(mode)`.
pub fn set_cli_teammate_mode_override(mode: TeammateMode) {
    TEAMMATE_MODE_SNAPSHOT
        .lock()
        .unwrap()
        .cli_teammate_mode_override = Some(mode);
}

/// Maps to: CC `getCliTeammateModeOverride()`.
pub fn get_cli_teammate_mode_override() -> Option<TeammateMode> {
    TEAMMATE_MODE_SNAPSHOT
        .lock()
        .unwrap()
        .cli_teammate_mode_override
}

/// Maps to: CC `clearCliTeammateModeOverride(newMode)`.
pub fn clear_cli_teammate_mode_override(new_mode: TeammateMode) {
    let mut state = TEAMMATE_MODE_SNAPSHOT.lock().unwrap();
    state.cli_teammate_mode_override = None;
    state.initial_teammate_mode = Some(new_mode);
}

/// Maps to: CC `captureTeammateModeSnapshot()`.
pub fn capture_teammate_mode_snapshot_from_config(config: &crate::utils::config::GlobalConfig) {
    let mut state = TEAMMATE_MODE_SNAPSHOT.lock().unwrap();
    state.initial_teammate_mode = Some(
        state
            .cli_teammate_mode_override
            .unwrap_or_else(|| mode_from_config(config)),
    );
}

/// Maps to: CC `captureTeammateModeSnapshot()`.
pub fn capture_teammate_mode_snapshot() {
    let config = crate::utils::config::load_global_config();
    capture_teammate_mode_snapshot_from_config(&config);
}

/// Maps to: CC `getTeammateModeFromSnapshot()`.
pub fn get_teammate_mode_from_snapshot() -> TeammateMode {
    if let Some(mode) = TEAMMATE_MODE_SNAPSHOT.lock().unwrap().initial_teammate_mode {
        return mode;
    }
    capture_teammate_mode_snapshot();
    TEAMMATE_MODE_SNAPSHOT
        .lock()
        .unwrap()
        .initial_teammate_mode
        .unwrap_or(TeammateMode::Auto)
}

/// Test/support reset for the module-level snapshot.
pub fn reset_teammate_mode_snapshot_for_test() {
    *TEAMMATE_MODE_SNAPSHOT.lock().unwrap() = TeammateModeSnapshotState::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teammate_mode_snapshot_uses_cli_override_then_config_like_official() {
        let _lock = crate::utils::swarm::backends::registry::TEST_BACKEND_REGISTRY_LOCK
            .lock()
            .unwrap();
        reset_teammate_mode_snapshot_for_test();
        let mut config = crate::utils::config::GlobalConfig::default();
        config.teammate_mode = Some("tmux".to_string());
        capture_teammate_mode_snapshot_from_config(&config);
        assert_eq!(get_teammate_mode_from_snapshot(), TeammateMode::Tmux);

        reset_teammate_mode_snapshot_for_test();
        set_cli_teammate_mode_override(TeammateMode::InProcess);
        capture_teammate_mode_snapshot_from_config(&config);
        assert_eq!(
            get_cli_teammate_mode_override(),
            Some(TeammateMode::InProcess)
        );
        assert_eq!(get_teammate_mode_from_snapshot(), TeammateMode::InProcess);

        clear_cli_teammate_mode_override(TeammateMode::Tmux);
        assert_eq!(get_cli_teammate_mode_override(), None);
        assert_eq!(get_teammate_mode_from_snapshot(), TeammateMode::Tmux);
        reset_teammate_mode_snapshot_for_test();
    }

    #[test]
    fn invalid_teammate_mode_defaults_to_auto_like_official_fallback() {
        let _lock = crate::utils::swarm::backends::registry::TEST_BACKEND_REGISTRY_LOCK
            .lock()
            .unwrap();
        reset_teammate_mode_snapshot_for_test();
        let mut config = crate::utils::config::GlobalConfig::default();
        config.teammate_mode = Some("unknown".to_string());
        capture_teammate_mode_snapshot_from_config(&config);
        assert_eq!(get_teammate_mode_from_snapshot(), TeammateMode::Auto);
        reset_teammate_mode_snapshot_for_test();
    }
}
