//! Human-readable auto-memory age and staleness reminders.
//!
//! Maps to: CC `memdir/memoryAge.ts:1-54`.

const DAY_MS: f64 = 86_400_000.0;

/// Clock-injected implementation detail for `memoryAgeDays`; production uses
/// [`memory_age_days`] so Date.now ownership remains at the source function.
fn memory_age_days_at(mtime_ms: f64, now_ms: f64) -> f64 {
    let days = ((now_ms - mtime_ms) / DAY_MS).floor();
    // JS Math.max propagates NaN; f64::max would silently turn it into zero.
    if days.is_nan() { days } else { days.max(0.0) }
}

/// Maps to: CC `memdir/memoryAge.ts:6-8` `memoryAgeDays`.
pub fn memory_age_days(mtime_ms: f64) -> f64 {
    memory_age_days_at(mtime_ms, chrono::Utc::now().timestamp_millis() as f64)
}

/// Maps to: CC `memdir/memoryAge.ts:15-20` `memoryAge`.
pub fn memory_age(mtime_ms: f64) -> String {
    match memory_age_days(mtime_ms) {
        0.0 => "today".to_string(),
        1.0 => "yesterday".to_string(),
        days => format!("{} days ago", ryu_js::Buffer::new().format(days)),
    }
}

/// Clock-injected implementation detail for `memoryFreshnessText`; production
/// uses [`memory_freshness_text`].
fn memory_freshness_text_at(mtime_ms: f64, now_ms: f64) -> String {
    let days = memory_age_days_at(mtime_ms, now_ms);
    if days <= 1.0 {
        return String::new();
    }
    let days = ryu_js::Buffer::new().format(days).to_string();
    format!(
        "This memory is {days} days old. Memories are point-in-time observations, not live state — claims about code behavior or file:line citations may be outdated. Verify against current code before asserting as fact."
    )
}

/// Maps to: CC `memdir/memoryAge.ts:33-43` `memoryFreshnessText`.
pub fn memory_freshness_text(mtime_ms: f64) -> String {
    memory_freshness_text_at(mtime_ms, chrono::Utc::now().timestamp_millis() as f64)
}

/// Maps to: CC `memdir/memoryAge.ts:49-53` `memoryFreshnessNote`.
pub fn memory_freshness_note(mtime_ms: f64) -> String {
    let text = memory_freshness_text(mtime_ms);
    if text.is_empty() {
        String::new()
    } else {
        format!("<system-reminder>{text}</system-reminder>\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_memory_time_matches_official_nan_propagation() {
        assert!(memory_age_days(f64::NAN).is_nan());
        assert_eq!(memory_age(f64::NAN), "NaN days ago");
        assert!(memory_freshness_text(f64::NAN).starts_with("This memory is NaN days old."));
    }

    #[test]
    fn memory_age_and_freshness_match_official_floor_and_future_clamp() {
        let now = 10.0 * DAY_MS;
        assert_eq!(memory_age_days_at(now, now), 0.0);
        assert_eq!(memory_age_days_at(now - DAY_MS, now), 1.0);
        assert_eq!(memory_age_days_at(now - 2.0 * DAY_MS - 1.0, now), 2.0);
        assert_eq!(memory_age_days_at(now + DAY_MS, now), 0.0);
        assert_eq!(memory_freshness_text_at(now - DAY_MS, now), "");
        assert_eq!(
            memory_freshness_text_at(now - 2.0 * DAY_MS, now),
            "This memory is 2 days old. Memories are point-in-time observations, not live state — claims about code behavior or file:line citations may be outdated. Verify against current code before asserting as fact."
        );
    }
}
