//! Extra-usage billing predicates.
//! Maps to CC `utils/extraUsage.ts`.

/// Maps to: CC `utils/extraUsage.ts:3-23` `isBilledAsExtraUsage`.
pub fn is_billed_as_extra_usage(
    model: Option<&str>,
    is_fast_mode: bool,
    is_opus_1m_merged: bool,
) -> bool {
    if !crate::utils::auth::is_claude_ai_subscriber() {
        return false;
    }
    if is_fast_mode {
        return true;
    }
    let Some(model) = model else {
        return false;
    };
    if !crate::utils::context::has_1m_context(model) {
        return false;
    }

    let m = model
        .to_ascii_lowercase()
        .trim_end_matches("[1m]")
        .trim()
        .to_string();
    let is_opus_46 = m == "opus" || m.contains("opus-4-6");
    let is_sonnet_46 = m == "sonnet" || m.contains("sonnet-4-6");

    if is_opus_46 && is_opus_1m_merged {
        return false;
    }

    is_opus_46 || is_sonnet_46
}

#[cfg(test)]
mod tests {
    use super::is_billed_as_extra_usage;

    #[test]
    fn extra_usage_stays_off_without_fast_mode_or_1m_tag() {
        assert!(!is_billed_as_extra_usage(None, false, false));
        assert!(!is_billed_as_extra_usage(Some("sonnet"), false, false));
        assert!(!is_billed_as_extra_usage(Some("opus"), false, true));
    }
}
