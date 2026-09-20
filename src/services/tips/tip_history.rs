//! Maps to: CC `services/tips/tipHistory.ts`.

use crate::utils::config::{load_global_config, save_global_config};

/// Maps to: CC `recordTipShown`.
pub fn record_tip_shown(tip_id: &str) {
    let num_startups = load_global_config().num_startups;
    let _ = save_global_config(|config| {
        let history = config.tips_history.get_or_insert_with(Default::default);
        if history.get(tip_id) == Some(&num_startups) {
            return;
        }
        history.insert(tip_id.to_string(), num_startups);
    });
}

/// Maps to: CC `getSessionsSinceLastShown`.
/// Returns `u64::MAX` when the tip has never been shown (CC `Infinity`).
pub fn get_sessions_since_last_shown(tip_id: &str) -> u64 {
    let config = load_global_config();
    let Some(last_shown) = config
        .tips_history
        .as_ref()
        .and_then(|history| history.get(tip_id).copied())
    else {
        return u64::MAX;
    };
    config.num_startups.saturating_sub(last_shown)
}
