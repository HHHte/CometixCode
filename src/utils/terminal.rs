//! Maps to: CC `utils/terminal.ts` — the transcript output-line helpers.
//! Only the truncation probe is ported; the wrapping renderer itself lives
//! with the components that consume it.

/// Maps to: CC `utils/terminal.ts:7` `MAX_LINES_TO_SHOW`.
const MAX_LINES_TO_SHOW: usize = 3;

/// Maps to: CC `utils/terminal.ts:119-131` `isOutputLineTruncated` — true
/// when the content holds more than `MAX_LINES_TO_SHOW` newlines ("the +1
/// accounts for wrapText showing an extra line when remainingLines==1").
pub fn is_output_line_truncated(content: &str) -> bool {
    let mut pos = 0usize;
    for _ in 0..=MAX_LINES_TO_SHOW {
        match content[pos..].find('\n') {
            None => return false,
            Some(offset) => pos += offset + 1,
        }
    }
    // A trailing newline is a terminator, not a new line — match
    // renderTruncatedContent's trimEnd() behavior.
    pos < content.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_probe_matches_official_newline_threshold() {
        assert!(!is_output_line_truncated("one line"));
        assert!(!is_output_line_truncated("a\nb\nc\nd"));
        assert!(is_output_line_truncated("a\nb\nc\nd\ne"));
        // CC :128-130: a trailing newline is a terminator, not a new line.
        assert!(!is_output_line_truncated("a\nb\nc\nd\n"));
        assert!(is_output_line_truncated("a\nb\nc\nd\ne\n"));
    }
}
