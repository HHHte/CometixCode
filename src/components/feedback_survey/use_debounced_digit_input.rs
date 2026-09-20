//! Maps to: CC `components/FeedbackSurvey/useDebouncedDigitInput.ts`.
//!
//! iocraft prompt input ownership lives outside this component tree. This file
//! ports the official digit-detection semantics as a pure helper so callers can
//! debounce and apply the returned trim/callback result without duplicating the
//! rules.

use crate::utils::string_utils::normalize_full_width_digits;

pub const DEFAULT_DEBOUNCE_MS: u64 = 400;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebouncedDigitCandidate<T> {
    pub trimmed_input: String,
    pub digit: T,
    pub debounce_ms: u64,
}

/// Maps to: CC `useDebouncedDigitInput(...)` digit extraction before the timer.
pub fn debounced_digit_candidate<T, F>(
    initial_input_value: &str,
    input_value: &str,
    enabled: bool,
    already_triggered: bool,
    once: bool,
    debounce_ms: Option<u64>,
    mut parse_digit: F,
) -> Option<DebouncedDigitCandidate<T>>
where
    F: FnMut(&str) -> Option<T>,
{
    if !enabled || (once && already_triggered) || input_value == initial_input_value {
        return None;
    }
    let last_char = input_value.chars().last()?.to_string();
    let normalized = normalize_full_width_digits(&last_char);
    let digit = parse_digit(&normalized)?;
    let trimmed_input = input_value
        .char_indices()
        .next_back()
        .map(|(idx, _)| input_value[..idx].to_string())
        .unwrap_or_default();
    Some(DebouncedDigitCandidate {
        trimmed_input,
        digit,
        debounce_ms: debounce_ms.unwrap_or(DEFAULT_DEBOUNCE_MS),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounced_digit_candidate_matches_official_full_width_and_once_rules() {
        let candidate = debounced_digit_candidate("", "abc３", true, false, false, None, |digit| {
            matches!(digit, "0" | "1" | "2" | "3").then(|| digit.to_string())
        })
        .expect("candidate");
        assert_eq!(candidate.trimmed_input, "abc");
        assert_eq!(candidate.digit, "3");
        assert_eq!(candidate.debounce_ms, DEFAULT_DEBOUNCE_MS);

        assert!(
            debounced_digit_candidate("same", "same", true, false, false, None, |digit| Some(
                digit.to_string()
            ))
            .is_none()
        );
        assert!(
            debounced_digit_candidate("", "1", false, false, false, None, |digit| Some(
                digit.to_string()
            ))
            .is_none()
        );
        assert!(
            debounced_digit_candidate("", "1", true, true, true, None, |digit| Some(
                digit.to_string()
            ))
            .is_none()
        );
    }
}
