//! Token budget prompt helpers.
//! Maps to: CC `utils/tokenBudget.ts`.

use std::sync::LazyLock;

static SHORTHAND_START_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)^\s*\+(\d+(?:\.\d+)?)\s*(k|m|b)\b").unwrap());
static SHORTHAND_END_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)\s\+(\d+(?:\.\d+)?)\s*(k|m|b)\s*[.!?]?\s*$").unwrap());
static VERBOSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b(?:use|spend)\s+(\d+(?:\.\d+)?)\s*(k|m|b)\s*tokens?\b").unwrap()
});
static VERBOSE_RE_G: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?i)\b(?:use|spend)\s+(\d+(?:\.\d+)?)\s*(k|m|b)\s*tokens?\b").unwrap()
});

/// Maps to CC `utils/tokenBudget.ts` `parseTokenBudget(...)`.
pub fn parse_token_budget(text: &str) -> Option<i64> {
    SHORTHAND_START_RE
        .captures(text)
        .or_else(|| SHORTHAND_END_RE.captures(text))
        .or_else(|| VERBOSE_RE.captures(text))
        .and_then(|captures| parse_budget_match(&captures[1], &captures[2]))
}

fn parse_budget_match(value: &str, suffix: &str) -> Option<i64> {
    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "k" => 1_000.0,
        "m" => 1_000_000.0,
        "b" => 1_000_000_000.0,
        _ => return None,
    };
    value
        .parse::<f64>()
        .ok()
        .map(|value| (value * multiplier) as i64)
}

/// Maps to CC `utils/tokenBudget.ts` `findTokenBudgetPositions(...)`.
pub fn find_token_budget_positions(text: &str) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();

    if let Some(start_match) = SHORTHAND_START_RE.find(text) {
        let matched = start_match.as_str();
        let leading_ws = matched.len().saturating_sub(matched.trim_start().len());
        let start = start_match.start() + leading_ws;
        positions.push((start, start_match.end()));
    }

    if let Some(end_match) = SHORTHAND_END_RE.find(text) {
        let end_start = end_match.start().saturating_add(1);
        let already_covered = positions
            .iter()
            .any(|(start, end)| end_start >= *start && end_start < *end);
        if !already_covered {
            positions.push((end_start, end_match.end()));
        }
    }

    for verbose_match in VERBOSE_RE_G.find_iter(text) {
        positions.push((verbose_match.start(), verbose_match.end()));
    }

    positions
}

/// Maps to CC `utils/tokenBudget.ts` `getBudgetContinuationMessage(...)`.
pub fn get_budget_continuation_message(pct: i64, turn_tokens: i64, budget: i64) -> String {
    format!(
        "Stopped at {pct}% of token target ({} / {}). Keep working — do not summarize.",
        format_number(turn_tokens),
        format_number(budget)
    )
}

fn format_number(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (idx, ch) in s.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let formatted = out.chars().rev().collect::<String>();
    if n < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_token_budget_matches_official_shorthand_and_verbose_forms() {
        assert_eq!(parse_token_budget("+500k keep going"), Some(500_000));
        assert_eq!(parse_token_budget("keep going +1.5m"), Some(1_500_000));
        assert_eq!(parse_token_budget("please use 2m tokens"), Some(2_000_000));
        assert_eq!(parse_token_budget("natural language only"), None);
    }

    #[test]
    fn find_token_budget_positions_matches_official_anchors() {
        assert_eq!(
            find_token_budget_positions("  +500k keep going"),
            vec![(2, 7)]
        );
        assert_eq!(
            find_token_budget_positions("keep going +1.5m"),
            vec![(11, 16)]
        );
        assert_eq!(
            find_token_budget_positions("please use 2m tokens"),
            vec![(7, 20)]
        );
    }

    #[test]
    fn get_budget_continuation_message_matches_official_copy() {
        assert_eq!(
            get_budget_continuation_message(12, 12_345, 100_000),
            "Stopped at 12% of token target (12,345 / 100,000). Keep working — do not summarize."
        );
    }
}
