//! Maps to: CC `components/Spinner/useShimmerAnimation.ts`.

use super::SpinnerMode;
use unicode_width::UnicodeWidthStr;

pub fn shimmer_animation_index(
    mode: SpinnerMode,
    message: &str,
    is_stalled: bool,
    time_ms: u128,
) -> isize {
    if is_stalled {
        return -100;
    }
    let speed = if mode == SpinnerMode::Requesting {
        50
    } else {
        200
    };
    let width = UnicodeWidthStr::width(message) as isize;
    let cycle = width + 20;
    let position = (time_ms / speed) as isize;
    if mode == SpinnerMode::Requesting {
        position.rem_euclid(cycle) - 10
    } else {
        width + 10 - position.rem_euclid(cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stalled_and_directional_indices_match_official() {
        assert_eq!(
            shimmer_animation_index(SpinnerMode::Requesting, "abc", true, 100),
            -100
        );
        assert_eq!(
            shimmer_animation_index(SpinnerMode::Requesting, "abc", false, 0),
            -10
        );
        assert_eq!(
            shimmer_animation_index(SpinnerMode::ToolUse, "abc", false, 0),
            13
        );
    }
}
