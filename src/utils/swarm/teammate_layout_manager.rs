//! Teammate color assignment and pane layout dispatch.
//!
//! Maps to: CC `utils/swarm/teammateLayoutManager.ts`.
//!
//! CC's file is two things: the per-session color assignment map, and four thin
//! dispatchers that resolve the detected pane backend (`getBackend()`, `:14-16`)
//! and forward one call to it. `handleSpawnSplitPane` imports the dispatchers
//! directly (`spawnMultiAgent.ts:59-65`) — it does NOT go through
//! `PaneBackendExecutor`.

use crate::tools::agent_tool::agent_color_manager::{AGENT_COLORS, AgentColorName};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use super::backends::detection;
use super::backends::iterm_backend::ITermBackend;
use super::backends::registry::detect_and_get_backend;
use super::backends::tmux_backend::{CreatePaneResult, TmuxBackend};
use super::backends::types::PaneBackendType;

#[derive(Default)]
struct TeammateColorState {
    assignments: HashMap<String, AgentColorName>,
    color_index: usize,
}

static TEAMMATE_COLOR_STATE: LazyLock<Mutex<TeammateColorState>> =
    LazyLock::new(|| Mutex::new(TeammateColorState::default()));

/// Maps to: CC `teammateLayoutManager.ts#assignTeammateColor`.
pub fn assign_teammate_color(teammate_id: &str) -> AgentColorName {
    let mut state = TEAMMATE_COLOR_STATE.lock().unwrap();
    if let Some(color) = state.assignments.get(teammate_id).copied() {
        return color;
    }
    let color = AGENT_COLORS[state.color_index % AGENT_COLORS.len()];
    state.assignments.insert(teammate_id.to_string(), color);
    state.color_index += 1;
    color
}

/// Maps to: CC `teammateLayoutManager.ts#getTeammateColor`.
pub fn get_teammate_color(teammate_id: &str) -> Option<AgentColorName> {
    TEAMMATE_COLOR_STATE
        .lock()
        .unwrap()
        .assignments
        .get(teammate_id)
        .copied()
}

/// Maps to: CC `teammateLayoutManager.ts#clearTeammateColors`.
pub fn clear_teammate_colors() {
    *TEAMMATE_COLOR_STATE.lock().unwrap() = TeammateColorState::default();
}

/// Maps to: CC `teammateLayoutManager.ts:57-60#isInsideTmux` — a re-export of
/// `backends/detection.ts#isInsideTmux` behind a dynamic import.
pub async fn is_inside_tmux() -> bool {
    detection::is_inside_tmux().await
}

/// Maps to: CC `teammateLayoutManager.ts:14-16#getBackend` —
/// `(await detectAndGetBackend()).backend`. Rust's detection result carries the
/// backend TYPE rather than a boxed backend object (the two concrete backends
/// are distinct structs, not one trait object), so each dispatcher below
/// re-matches on it. Same single detection call, same caching.
async fn get_backend() -> Result<PaneBackendType, String> {
    Ok(detect_and_get_backend().await?.backend_type)
}

/// Maps to: CC `teammateLayoutManager.ts:76-82#createTeammatePaneInSwarmView`.
pub async fn create_teammate_pane_in_swarm_view(
    teammate_name: &str,
    teammate_color: AgentColorName,
) -> Result<CreatePaneResult, String> {
    match get_backend().await? {
        PaneBackendType::Tmux => {
            TmuxBackend::new()
                .create_teammate_pane_in_swarm_view(teammate_name, teammate_color)
                .await
        }
        PaneBackendType::ITerm2 => {
            ITermBackend::new()
                .create_teammate_pane_in_swarm_view(teammate_name, teammate_color)
                .await
        }
    }
}

/// Maps to: CC `teammateLayoutManager.ts:88-94#enablePaneBorderStatus`.
pub async fn enable_pane_border_status(
    window_target: Option<&str>,
    use_swarm_socket: bool,
) -> Result<(), String> {
    match get_backend().await? {
        PaneBackendType::Tmux => {
            TmuxBackend::new()
                .enable_pane_border_status(window_target, use_swarm_socket)
                .await
        }
        PaneBackendType::ITerm2 => {
            ITermBackend::new()
                .enable_pane_border_status(window_target, use_swarm_socket)
                .await
        }
    }
}

/// Maps to: CC `teammateLayoutManager.ts:100-107#sendCommandToPane`.
pub async fn send_command_to_pane(
    pane_id: &str,
    command: &str,
    use_swarm_socket: bool,
) -> Result<(), String> {
    match get_backend().await? {
        PaneBackendType::Tmux => {
            TmuxBackend::new()
                .send_command_to_pane(pane_id, command, use_swarm_socket)
                .await
        }
        PaneBackendType::ITerm2 => {
            ITermBackend::new()
                .send_command_to_pane(pane_id, command, use_swarm_socket)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn assigns_teammate_colors_round_robin_and_reuses_existing_assignment() {
        super::clear_teammate_colors();
        let first = super::assign_teammate_color("a@team");
        let second = super::assign_teammate_color("b@team");
        assert_ne!(first, second);
        assert_eq!(super::assign_teammate_color("a@team"), first);
        assert_eq!(super::get_teammate_color("b@team"), Some(second));
    }
}
