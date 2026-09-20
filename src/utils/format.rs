/// Rust numeric carrier for CC's JavaScript `number` parameter.
pub trait FileSizeNumber {
    fn to_file_size_number(self) -> f64;
}

macro_rules! impl_file_size_number {
    ($($number:ty),+ $(,)?) => {
        $(
            impl FileSizeNumber for $number {
                fn to_file_size_number(self) -> f64 {
                    self as f64
                }
            }
        )+
    };
}

impl_file_size_number!(
    f64, f32, u128, u64, usize, u32, u16, u8, i128, i64, isize, i32, i16, i8,
);

/// Maps to: CC `utils/format.ts#formatFileSize(sizeInBytes)`.
pub fn format_file_size(bytes: impl FileSizeNumber) -> String {
    let bytes = bytes.to_file_size_number();
    let kb = bytes / 1024.0;
    if kb < 1.0 {
        return format!("{} bytes", ryu_js::Buffer::new().format(bytes));
    }
    if kb < 1024.0 {
        return format!("{}KB", strip_trailing_dot_zero(kb));
    }
    let mb = kb / 1024.0;
    if mb < 1024.0 {
        return format!("{}MB", strip_trailing_dot_zero(mb));
    }
    let gb = mb / 1024.0;
    format!("{}GB", strip_trailing_dot_zero(gb))
}

fn strip_trailing_dot_zero(value: f64) -> String {
    let mut buffer = ryu_js::Buffer::new();
    let rendered = buffer.format_to_fixed(value, 1);
    rendered.strip_suffix(".0").unwrap_or(rendered).to_string()
}

/// Maps to: CC `utils/format.ts:30-32` `formatSecondsShort`.
pub fn format_seconds_short(ms: u64) -> String {
    format!(
        "{}s",
        ryu_js::Buffer::new().format_to_fixed(ms as f64 / 1000.0, 1)
    )
}

/// Maps to: CC `utils/format.ts:124-132` `formatNumber`.
pub fn format_number(number: u64) -> String {
    if number < 1_000 {
        return number.to_string();
    }

    let units = [
        (1_000_f64, "k"),
        (1_000_000_f64, "m"),
        (1_000_000_000_f64, "b"),
    ];
    let mut unit_index = units
        .iter()
        .rposition(|(threshold, _)| number as f64 >= *threshold)
        .unwrap_or(0);
    let mut rounded_tenths = ((number as f64 / units[unit_index].0) * 10.0 + 0.5).floor() as u64;
    if rounded_tenths >= 10_000 && unit_index + 1 < units.len() {
        unit_index += 1;
        rounded_tenths = ((number as f64 / units[unit_index].0) * 10.0 + 0.5).floor() as u64;
    }
    format!(
        "{}.{}{}",
        rounded_tenths / 10,
        rounded_tenths % 10,
        units[unit_index].1
    )
}

/// Maps to: CC `utils/format.ts` `formatTokens`.
pub fn format_tokens(count: u64) -> String {
    format_number(count).replace(".0", "")
}

/// Maps to: CC `utils/format.ts:34-99` `formatDuration`.
pub fn format_duration(ms: u64) -> String {
    if ms < 60_000 {
        if ms == 0 {
            return "0s".to_string();
        }
        if ms < 1 {
            return format!("{:.1}s", ms as f64 / 1000.0);
        }
        return format!("{}s", ms / 1000);
    }

    let mut days = ms / 86_400_000;
    let mut hours = (ms % 86_400_000) / 3_600_000;
    let mut minutes = (ms % 3_600_000) / 60_000;
    let mut seconds = ((ms % 60_000) + 500) / 1000;

    if seconds == 60 {
        seconds = 0;
        minutes += 1;
    }
    if minutes == 60 {
        minutes = 0;
        hours += 1;
    }
    if hours == 24 {
        hours = 0;
        days += 1;
    }

    if days > 0 {
        return format!("{days}d {hours}h {minutes}m");
    }
    if hours > 0 {
        return format!("{hours}h {minutes}m {seconds}s");
    }
    if minutes > 0 {
        return format!("{minutes}m {seconds}s");
    }
    format!("{seconds}s")
}

/// Maps to: CC `utils/format.ts:186-197` `formatRelativeTimeAgo` with the
/// default narrow style.
pub fn format_relative_time_ago(time: std::time::SystemTime) -> String {
    fn millis(time: std::time::SystemTime) -> i64 {
        match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as i64,
            Err(error) => -(error.duration().as_millis() as i64),
        }
    }
    format_relative_time_ago_millis(millis(time), millis(std::time::SystemTime::now()))
}

/// Maps to: CC `utils/format.ts:144-184` `formatRelativeTime` with the
/// source-default narrow/always options. Rust represents the two Date values
/// as epoch milliseconds at this pure formatting boundary.
pub fn format_relative_time(timestamp_ms: i64, now_ms: i64) -> String {
    let diff_seconds = (timestamp_ms - now_ms) / 1000;
    let intervals = [
        (31_536_000_i64, "y"),
        (2_592_000_i64, "mo"),
        (604_800_i64, "w"),
        (86_400_i64, "d"),
        (3_600_i64, "h"),
        (60_i64, "m"),
        (1_i64, "s"),
    ];

    for (interval, short_unit) in intervals {
        if diff_seconds.abs() >= interval {
            let value = diff_seconds / interval;
            return if diff_seconds < 0 {
                format!("{}{} ago", value.abs(), short_unit)
            } else {
                format!("in {}{}", value, short_unit)
            };
        }
    }

    if diff_seconds <= 0 {
        "0s ago".to_string()
    } else {
        "in 0s".to_string()
    }
}

/// Maps to: CC `utils/format.ts:186-197` `formatRelativeTimeAgo`; Rust injects
/// `now` as milliseconds for deterministic callers/tests and delegates to the
/// canonical `formatRelativeTime` owner.
pub fn format_relative_time_ago_millis(timestamp_ms: i64, now_ms: i64) -> String {
    format_relative_time(timestamp_ms, now_ms)
}

/// Maps to: CC `formatRelativeTimeAgo(date, { style: 'short' })` — the
/// `formatLogMetadata` call (`utils/format.ts:210-218`).
///
/// Every non-narrow style renders through the Intl `long` formatter — CC's
/// own comment: "For days and longer, use long style regardless of the style
/// parameter" (`utils/format.ts:174-176`) — so `short` yields "2 hours ago",
/// not "2h ago".
pub fn format_relative_time_ago_short(time: std::time::SystemTime) -> String {
    fn millis(time: std::time::SystemTime) -> i64 {
        match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as i64,
            Err(error) => -(error.duration().as_millis() as i64),
        }
    }
    format_relative_time_ago_short_millis(millis(time), millis(std::time::SystemTime::now()))
}

/// Maps to: CC `utils/format.ts:144-184` `formatRelativeTime` non-narrow
/// branch with `numeric: 'always'` (the only numeric mode the ago wrappers
/// use): `Intl.RelativeTimeFormat('long').format(value, unit)`.
pub fn format_relative_time_ago_short_millis(timestamp_ms: i64, now_ms: i64) -> String {
    let diff_seconds = (timestamp_ms - now_ms) / 1000;
    let intervals = [
        (31_536_000_i64, "year"),
        (2_592_000_i64, "month"),
        (604_800_i64, "week"),
        (86_400_i64, "day"),
        (3_600_i64, "hour"),
        (60_i64, "minute"),
        (1_i64, "second"),
    ];

    for (interval, unit) in intervals {
        if diff_seconds.abs() >= interval {
            let value = (diff_seconds / interval).abs();
            let plural = if value == 1 { "" } else { "s" };
            return if diff_seconds < 0 {
                format!("{value} {unit}{plural} ago")
            } else {
                format!("in {value} {unit}{plural}")
            };
        }
    }

    // CC's sub-second branch calls `.format(0, 'second')`, which Intl with
    // `numeric: 'always'` renders as "in 0 seconds" even for the past.
    "in 0 seconds".to_string()
}

/// Maps to: CC `utils/format.ts:203-235` `formatLogMetadata`.
///
/// Callers pass whichever size/count metadata exists on their log record. The
/// optional fields preserve the source's omission semantics while keeping the
/// formatting owner shared by resume selectors and log views.
pub fn format_log_metadata(
    modified: std::time::SystemTime,
    message_count: Option<usize>,
    file_size: Option<u64>,
    git_branch: Option<&str>,
    tag: Option<&str>,
    agent_setting: Option<&str>,
    pr_number: Option<u64>,
    pr_repository: Option<&str>,
) -> String {
    let size_or_count = file_size
        .map(format_file_size)
        .or_else(|| message_count.map(|count| format!("{count} messages")))
        .unwrap_or_else(|| "0 messages".to_string());
    let mut parts = vec![format_relative_time_ago_short(modified)];
    if let Some(branch) = git_branch.filter(|branch| !branch.is_empty()) {
        parts.push(branch.to_string());
    }
    parts.push(size_or_count);
    if let Some(tag) = tag.filter(|tag| !tag.is_empty()) {
        parts.push(format!("#{tag}"));
    }
    if let Some(agent) = agent_setting.filter(|agent| !agent.is_empty()) {
        parts.push(format!("@{agent}"));
    }
    if let Some(number) = pr_number {
        parts.push(match pr_repository.filter(|repo| !repo.is_empty()) {
            Some(repository) => format!("{repository}#{number}"),
            None => format!("#{number}"),
        });
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_size_matches_official_format() {
        assert_eq!(format_file_size(512), "512 bytes");
        assert_eq!(format_file_size(0.5), "0.5 bytes");
        assert_eq!(format_file_size(1536), "1.5KB");
        assert_eq!(format_file_size(2304), "2.3KB");
        assert_eq!(format_file_size(1024 * 1024), "1MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1GB");
    }

    #[test]
    fn format_seconds_short_matches_official_to_fixed() {
        assert_eq!(format_seconds_short(0), "0.0s");
        assert_eq!(format_seconds_short(1_234), "1.2s");
        assert_eq!(format_seconds_short(1_250), "1.3s");
    }

    #[test]
    fn format_number_matches_official_compact_format() {
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1_000), "1.0k");
        assert_eq!(format_number(12_500), "12.5k");
        assert_eq!(format_number(999_999), "1.0m");
        assert_eq!(format_number(1_250_000), "1.3m");
    }

    #[test]
    fn format_tokens_strips_official_trailing_zero_decimal() {
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1_000), "1k");
        assert_eq!(format_tokens(12_500), "12.5k");
        assert_eq!(format_tokens(1_000_000), "1m");
    }

    #[test]
    fn format_duration_matches_official_default_format() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(4_200), "4s");
        assert_eq!(format_duration(119_500), "2m 0s");
        assert_eq!(format_duration(3_661_000), "1h 1m 1s");
        assert_eq!(format_duration(90_061_000), "1d 1h 1m");
    }

    #[test]
    fn format_relative_time_ago_millis_matches_official_narrow_shape() {
        let now = 1_700_000_000_000_i64;
        assert_eq!(format_relative_time_ago_millis(now, now), "0s ago");
        assert_eq!(
            format_relative_time_ago_millis(now - 59_000, now),
            "59s ago"
        );
        assert_eq!(format_relative_time_ago_millis(now - 60_000, now), "1m ago");
        assert_eq!(
            format_relative_time_ago_millis(now - 3_600_000, now),
            "1h ago"
        );
        assert_eq!(format_relative_time_ago_millis(now + 60_000, now), "in 1m");
    }

    /// CC `formatLogMetadata` passes `{ style: 'short' }`, and every
    /// non-narrow style renders Intl `long` copy (`utils/format.ts:174-176`).
    #[test]
    fn format_relative_time_ago_short_matches_official_long_copy() {
        let now = 1_700_000_000_000_i64;
        assert_eq!(
            format_relative_time_ago_short_millis(now - 21 * 60_000, now),
            "21 minutes ago"
        );
        assert_eq!(
            format_relative_time_ago_short_millis(now - 3_600_000, now),
            "1 hour ago"
        );
        assert_eq!(
            format_relative_time_ago_short_millis(now - 2 * 3_600_000, now),
            "2 hours ago"
        );
        assert_eq!(
            format_relative_time_ago_short_millis(now - 56 * 31_536_000_000, now),
            "56 years ago"
        );
        assert_eq!(
            format_relative_time_ago_short_millis(now + 2 * 3_600_000, now),
            "in 2 hours"
        );
        assert_eq!(
            format_relative_time_ago_short_millis(now, now),
            "in 0 seconds"
        );
    }
}
