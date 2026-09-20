//! Maps to: CC `components/PromptInput/useShowFastIconHint.ts:1-31`.

use iocraft::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const HINT_DISPLAY_DURATION: Duration = Duration::from_millis(5_000);
static HAS_SHOWN_THIS_SESSION: AtomicBool = AtomicBool::new(false);

pub fn should_show_fast_icon_hint(show_fast_icon: bool, claimed: bool, elapsed: Duration) -> bool {
    show_fast_icon && claimed && elapsed < HINT_DISPLAY_DURATION
}

pub fn use_show_fast_icon_hint(hooks: &mut Hooks, show_fast_icon: bool) -> bool {
    let mut started = hooks.use_state(|| None::<Instant>);
    // Mirrors the effect cleanup when the icon disappears: cancel the active
    // hint, but keep the process-wide once-per-session claim consumed.
    if !show_fast_icon && started.read().is_some() {
        started.set(None);
    }
    if show_fast_icon
        && started.read().is_none()
        && HAS_SHOWN_THIS_SESSION
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    {
        started.set(Some(Instant::now()));
    }
    let active = started
        .read()
        .is_some_and(|start| start.elapsed() < HINT_DISPLAY_DURATION);
    let _frame = hooks.use_animation_frame(active.then_some(Duration::from_millis(100)));
    active && show_fast_icon
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pure_duration_gate_expires_at_five_seconds() {
        assert!(should_show_fast_icon_hint(
            true,
            true,
            Duration::from_millis(4_999)
        ));
        assert!(!should_show_fast_icon_hint(
            true,
            true,
            Duration::from_millis(5_000)
        ));
        assert!(!should_show_fast_icon_hint(false, true, Duration::ZERO));
    }
}
