//! Maps to: CC `utils/autoModeDenials.ts`.

use crate::utils::permissions::permission_setup::is_transcript_classifier_feature_enabled;
use std::sync::{Arc, LazyLock, Mutex};

/// Maps to: CC `utils/autoModeDenials.ts#AutoModeDenial:8-14`.
#[derive(Clone, Debug, PartialEq)]
pub struct AutoModeDenial {
    pub tool_name: String,
    pub display: String,
    pub reason: String,
    pub timestamp: f64,
}

// CC :16 replaces its readonly array on record; Arc preserves a mounted
// reader's old array when a new denial is prepended.
static DENIALS: LazyLock<Mutex<Arc<Vec<AutoModeDenial>>>> =
    LazyLock::new(|| Mutex::new(Arc::new(Vec::new())));
const MAX_DENIALS: usize = 20;

/// Maps to: CC `utils/autoModeDenials.ts#recordAutoModeDenial:19-22`.
pub fn record_auto_mode_denial(denial: AutoModeDenial) {
    if !is_transcript_classifier_feature_enabled() {
        return;
    }
    let mut denials = DENIALS.lock().unwrap_or_else(|error| error.into_inner());
    *denials = Arc::new(
        std::iter::once(denial)
            .chain(denials.iter().take(MAX_DENIALS - 1).cloned())
            .collect(),
    );
}

/// Maps to: CC `utils/autoModeDenials.ts#getAutoModeDenials:24-26`.
pub fn get_auto_mode_denials() -> Arc<Vec<AutoModeDenial>> {
    DENIALS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_mode_denials_matches_official_prepend_limit_and_snapshot_identity() {
        // CC autoModeDenials.ts:16-26: get returns the current array; record
        // replaces it, prepends without deduplication, and keeps at most 20.
        let initial = get_auto_mode_denials();
        assert!(Arc::ptr_eq(&initial, &get_auto_mode_denials()));
        for index in 0..22 {
            record_auto_mode_denial(AutoModeDenial {
                tool_name: "Bash".into(),
                display: "same command".into(),
                reason: format!("reason-{index}"),
                timestamp: index as f64,
            });
        }
        let current = get_auto_mode_denials();
        assert_eq!(current.len(), 20);
        assert_eq!(current[0].timestamp, 21.0);
        assert_eq!(current[19].timestamp, 2.0);
        assert_eq!(current[0].tool_name, "Bash");
        assert_eq!(current[0].reason, "reason-21");
        assert!(!Arc::ptr_eq(&initial, &current));
        assert!(initial.is_empty());
        assert!(Arc::ptr_eq(&current, &get_auto_mode_denials()));
    }
}
