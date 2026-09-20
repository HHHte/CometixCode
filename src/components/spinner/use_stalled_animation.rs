//! Maps to: CC `components/Spinner/useStalledAnimation.ts`.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StalledAnimationOutput {
    pub is_stalled: bool,
    pub stalled_intensity: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StalledAnimationState {
    mount_time: u64,
    last_token_time: u64,
    last_response_length: usize,
    smoothed_intensity: f32,
    last_smooth_time: u64,
}

impl StalledAnimationState {
    pub fn new(time_ms: u64, response_length: usize) -> Self {
        Self {
            mount_time: time_ms,
            last_token_time: time_ms,
            last_response_length: response_length,
            smoothed_intensity: 0.0,
            last_smooth_time: time_ms,
        }
    }

    pub fn update(
        &mut self,
        time_ms: u64,
        response_length: usize,
        has_active_tools: bool,
        reduced_motion: bool,
    ) -> StalledAnimationOutput {
        if response_length > self.last_response_length {
            self.last_token_time = time_ms;
            self.last_response_length = response_length;
            self.smoothed_intensity = 0.0;
            self.last_smooth_time = time_ms;
        }
        let elapsed = if has_active_tools {
            self.last_token_time = time_ms;
            0
        } else if response_length > 0 {
            time_ms.saturating_sub(self.last_token_time)
        } else {
            time_ms.saturating_sub(self.mount_time)
        };
        let is_stalled = elapsed > 3_000 && !has_active_tools;
        let target = if is_stalled {
            ((elapsed.saturating_sub(3_000)) as f32 / 2_000.0).min(1.0)
        } else {
            0.0
        };
        if !reduced_motion && (target > 0.0 || self.smoothed_intensity > 0.0) {
            let steps = time_ms.saturating_sub(self.last_smooth_time) / 50;
            for _ in 0..steps {
                let difference = target - self.smoothed_intensity;
                if difference.abs() < 0.01 {
                    self.smoothed_intensity = target;
                    break;
                }
                self.smoothed_intensity += difference * 0.1;
            }
            if steps > 0 {
                self.last_smooth_time = time_ms;
            }
        } else {
            self.smoothed_intensity = target;
            self.last_smooth_time = time_ms;
        }
        StalledAnimationOutput {
            is_stalled,
            stalled_intensity: if reduced_motion {
                target
            } else {
                self.smoothed_intensity
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stall_begins_after_three_seconds_and_resets_on_tokens() {
        let mut state = StalledAnimationState::new(0, 0);
        assert!(!state.update(3_000, 0, false, true).is_stalled);
        assert_eq!(state.update(5_000, 0, false, true).stalled_intensity, 1.0);
        assert_eq!(state.update(5_100, 1, false, true).stalled_intensity, 0.0);
    }
}
