//! Maps to: CC `utils/permissions/denialTracking.ts`.
//!
//! Pure denial counters used by classifier permission flows. No UI or runtime
//! side effects live here, matching the official utility boundary.

/// Maps to: CC `DenialTrackingState`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DenialTrackingState {
    pub consecutive_denials: u32,
    pub total_denials: u32,
}

/// Maps to: CC `DENIAL_LIMITS`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DenialLimits {
    pub max_consecutive: u32,
    pub max_total: u32,
}

pub const DENIAL_LIMITS: DenialLimits = DenialLimits {
    max_consecutive: 3,
    max_total: 20,
};

/// Maps to: CC `createDenialTrackingState()`.
pub fn create_denial_tracking_state() -> DenialTrackingState {
    DenialTrackingState {
        consecutive_denials: 0,
        total_denials: 0,
    }
}

/// Maps to: CC `recordDenial(state)`.
pub fn record_denial(state: DenialTrackingState) -> DenialTrackingState {
    DenialTrackingState {
        consecutive_denials: state.consecutive_denials + 1,
        total_denials: state.total_denials + 1,
    }
}

/// Maps to: CC `recordSuccess(state)`.
pub fn record_success(state: DenialTrackingState) -> DenialTrackingState {
    if state.consecutive_denials == 0 {
        state
    } else {
        DenialTrackingState {
            consecutive_denials: 0,
            ..state
        }
    }
}

/// Maps to: CC `shouldFallbackToPrompting(state)`.
pub fn should_fallback_to_prompting(state: DenialTrackingState) -> bool {
    state.consecutive_denials >= DENIAL_LIMITS.max_consecutive
        || state.total_denials >= DENIAL_LIMITS.max_total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denial_tracking_matches_official_limits_and_reset_behavior() {
        let mut state = create_denial_tracking_state();
        assert!(!should_fallback_to_prompting(state));

        state = record_denial(state);
        state = record_denial(state);
        assert_eq!(state.consecutive_denials, 2);
        assert_eq!(state.total_denials, 2);
        assert!(!should_fallback_to_prompting(state));

        state = record_denial(state);
        assert!(should_fallback_to_prompting(state));

        state = record_success(state);
        assert_eq!(state.consecutive_denials, 0);
        assert_eq!(state.total_denials, 3);
    }

    #[test]
    fn total_denial_limit_triggers_even_after_successes() {
        let mut state = create_denial_tracking_state();
        for _ in 0..DENIAL_LIMITS.max_total {
            state = record_success(record_denial(state));
        }
        assert_eq!(state.consecutive_denials, 0);
        assert_eq!(state.total_denials, DENIAL_LIMITS.max_total);
        assert!(should_fallback_to_prompting(state));
    }
}
