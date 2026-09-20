//! Frontmatter parser for markdown files.
//! Maps to: CC `utils/frontmatterParser.ts`.
//!
//! This module ports the frontmatter surface needed by output-style loading:
//! YAML frontmatter extraction, scalar description coercion, and markdown
//! description fallback. Broader skill/command helpers remain in their
//! dedicated future slices.

use serde_json::Value;
use std::collections::BTreeMap;

/// Maps to: CC `utils/frontmatterParser.ts:123#FRONTMATTER_REGEX`.
/// ECMAScript whitespace excludes U+0085 and includes U+FEFF. The closing
/// delimiter is deliberately not line-anchored, as in the original regex.
pub static FRONTMATTER_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"^---[\t-\r \u{00a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]*\n([\s\S]*?)---[\t-\r \u{00a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}]*\n?")
        .expect("source frontmatter regex compiles")
});

pub type FrontmatterData = BTreeMap<String, Value>;

/// Maps to: CC `utils/frontmatterParser.ts` `FrontmatterShell`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontmatterShell {
    Bash,
    PowerShell,
}

impl FrontmatterShell {
    pub fn official_name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::PowerShell => "powershell",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedMarkdown {
    pub frontmatter: FrontmatterData,
    pub content: String,
}

fn yaml_value_to_json(value: serde_yaml::Value) -> Value {
    match value {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(value) => Value::Bool(value),
        serde_yaml::Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Value::Number(value.into())
            } else if let Some(value) = number.as_u64() {
                Value::Number(value.into())
            } else if let Some(value) = number.as_f64() {
                serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(value) => Value::String(value),
        serde_yaml::Value::Sequence(values) => {
            Value::Array(values.into_iter().map(yaml_value_to_json).collect())
        }
        serde_yaml::Value::Mapping(mapping) => {
            let mut object = serde_json::Map::new();
            for (key, value) in mapping {
                if let serde_yaml::Value::String(key) = key {
                    object.insert(key, yaml_value_to_json(value));
                }
            }
            Value::Object(object)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_json(tagged.value),
    }
}

fn yaml_object_to_frontmatter(value: serde_yaml::Value) -> Option<FrontmatterData> {
    match yaml_value_to_json(value) {
        Value::Object(object) => Some(object.into_iter().collect()),
        _ => None,
    }
}

fn parse_yaml_mapping(text: &str) -> Option<FrontmatterData> {
    crate::utils::yaml::parse_yaml(text)
        .ok()
        .and_then(yaml_object_to_frontmatter)
}

fn has_yaml_special_chars(value: &str) -> bool {
    value.contains(": ")
        || value.chars().any(|ch| {
            matches!(
                ch,
                '{' | '}' | '[' | ']' | '*' | '&' | '#' | '!' | '|' | '>' | '%' | '@' | '`'
            )
        })
}

fn quote_problematic_values(frontmatter_text: &str) -> String {
    // Maps to CC `utils/frontmatterParser.ts#quoteProblematicValues`.
    frontmatter_text
        .split('\n')
        .map(|line| {
            let Some((key, value)) = line.split_once(':') else {
                return line.to_string();
            };
            if key.is_empty()
                || !key
                    .chars()
                    .all(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '-')
                || !value.starts_with(char::is_whitespace)
            {
                return line.to_string();
            }
            let value = value.trim_start();
            if value.is_empty()
                || ((value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\'')))
                || !has_yaml_special_chars(value)
            {
                return line.to_string();
            }
            let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
            format!("{key}: \"{escaped}\"")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_frontmatter_mapping_with_retry(frontmatter_text: &str) -> FrontmatterData {
    parse_yaml_mapping(frontmatter_text)
        .or_else(|| parse_yaml_mapping(&quote_problematic_values(frontmatter_text)))
        .unwrap_or_default()
}

/// Maps to: CC `utils/frontmatterParser.ts:130-177#parseFrontmatter`.
pub fn parse_frontmatter(markdown: &str) -> ParsedMarkdown {
    let Some(captures) = FRONTMATTER_REGEX.captures(markdown) else {
        return ParsedMarkdown {
            frontmatter: FrontmatterData::new(),
            content: markdown.to_string(),
        };
    };
    let frontmatter_text = captures.get(1).map_or("", |text| text.as_str());
    let content = &markdown[captures.get(0).expect("whole regex match").end()..];
    ParsedMarkdown {
        frontmatter: parse_frontmatter_mapping_with_retry(frontmatter_text),
        content: content.to_string(),
    }
}

/// Maps to: CC `utils/frontmatterParser.ts:189-232#splitPathInFrontmatter`.
///
/// Splits a comma-separated frontmatter scalar and expands brace patterns.
/// Commas inside braces are not separators. A YAML list is accepted too, and
/// each element goes through the same split+expand (CC's
/// `input.flatMap(splitPathInFrontmatter)` recursion, so nested lists work).
/// Any other JSON shape yields `[]` (CC's `typeof input !== 'string'` guard).
///
/// The brace-depth counter is signed on purpose: CC decrements past zero on a
/// stray `}`, and a comma at negative depth then does NOT split. A
/// `saturating_sub` clamp would split `a},b` into two patterns where CC keeps
/// one.
pub fn split_path_in_frontmatter(input: &Value) -> Vec<String> {
    let input = match input {
        Value::Array(values) => {
            return values.iter().flat_map(split_path_in_frontmatter).collect();
        }
        Value::String(value) => value.as_str(),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => return Vec::new(),
    };

    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut brace_depth: i64 = 0;

    for character in input.chars() {
        match character {
            '{' => {
                brace_depth += 1;
                current.push(character);
            }
            '}' => {
                brace_depth -= 1;
                current.push(character);
            }
            ',' if brace_depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(character),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }

    parts
        .iter()
        .filter(|part| !part.is_empty())
        .flat_map(|part| expand_braces(part))
        .collect()
}

/// Maps to: CC `utils/frontmatterParser.ts:240-266#expandBraces`.
///
/// CC matches `/^([^{]*)\{([^}]+)\}(.*)$/` — the FIRST `{`, then the FIRST `}`
/// after it, with at least one character between them — and recurses on each
/// combined alternative so later brace groups expand too.
///
/// `[^}]+` is why `a{}b` is returned untouched instead of collapsing to `ab`:
/// an empty alternative list does not match, so the pattern has "no braces".
///
/// This is bash-style alternation only, NOT full brace expansion: there are no
/// ranges (`{1..3}` stays literal) and nesting is not recognised — the
/// alternative list stops at the first `}` and the rest becomes the suffix, so
/// `{a,{b,c}}` yields `["a}", "b", "c}"]`. Both behaviours are CC's, confirmed
/// by running its regex directly.
///
/// Deliberate simplification: CC's `(.*)$` also fails to match when the text
/// after the closing brace contains an interior newline. Unreachable here —
/// `split_path_in_frontmatter` trims every comma-part and glob patterns are
/// single-line.
fn expand_braces(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };
    let Some(relative_close) = pattern[open + 1..].find('}') else {
        return vec![pattern.to_string()];
    };
    let close = open + 1 + relative_close;
    if close == open + 1 {
        // Empty alternative list: CC's `[^}]+` requires one or more characters.
        return vec![pattern.to_string()];
    }

    let prefix = &pattern[..open];
    let suffix = &pattern[close + 1..];
    pattern[open + 1..close]
        .split(',')
        .flat_map(|alternative| expand_braces(&format!("{prefix}{}{suffix}", alternative.trim())))
        .collect()
}

/// Maps to: CC `utils/frontmatterParser.ts#parsePositiveIntFromFrontmatter`.
pub fn parse_positive_int_from_frontmatter(value: Option<&Value>) -> Option<u32> {
    let parsed = match value? {
        Value::Number(number) => {
            let number = number.as_f64()?;
            if !number.is_finite() || number.fract() != 0.0 || number <= 0.0 {
                return None;
            }
            number as u64
        }
        Value::String(value) => {
            let mut chars = value.trim_start().chars().peekable();
            let negative = match chars.peek().copied() {
                Some('+') => {
                    chars.next();
                    false
                }
                Some('-') => {
                    chars.next();
                    true
                }
                _ => false,
            };
            if negative {
                return None;
            }
            let digits = chars
                .take_while(|character| character.is_ascii_digit())
                .collect::<String>();
            if digits.is_empty() {
                return None;
            }
            digits.parse::<u64>().ok()?
        }
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => return None,
    };

    u32::try_from(parsed).ok().filter(|parsed| *parsed > 0)
}

/// Maps to: CC `utils/frontmatterParser.ts#coerceDescriptionToString`.
pub fn coerce_description_to_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::Null => None,
        Value::String(value) => {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        }
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Array(_) | Value::Object(_) => None,
    }
}

/// Maps to: CC `utils/frontmatterParser.ts:332-334#parseBooleanFrontmatter`.
pub fn parse_boolean_frontmatter(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::Bool(true)))
        || matches!(value, Some(Value::String(text)) if text == "true")
}

/// Maps to: CC `utils/frontmatterParser.ts#parseShellFrontmatter`.
pub fn parse_shell_frontmatter(value: Option<&Value>, _source: &str) -> Option<FrontmatterShell> {
    let value = value?;
    if value.is_null() {
        return None;
    }
    let normalized = match value {
        Value::String(value) => value.trim().to_ascii_lowercase(),
        Value::Number(value) => value.to_string().trim().to_ascii_lowercase(),
        Value::Bool(value) => value.to_string().trim().to_ascii_lowercase(),
        Value::Null | Value::Array(_) | Value::Object(_) => String::new(),
    };
    match normalized.as_str() {
        "" => None,
        "bash" => Some(FrontmatterShell::Bash),
        "powershell" => Some(FrontmatterShell::PowerShell),
        // Official logs and falls back to bash for unrecognized values. This
        // Rust port has no debug logger dependency in the parser boundary, so
        // it preserves the user-visible fallback behavior without logging.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_boolean_frontmatter_matches_official_literal_values() {
        // CC frontmatterParser.ts:332-334: value === true || value === 'true'.
        for value in [serde_json::json!(true), serde_json::json!("true")] {
            assert!(parse_boolean_frontmatter(Some(&value)));
        }
        for value in [
            serde_json::json!(false),
            serde_json::json!("TRUE"),
            serde_json::json!(" true "),
            serde_json::json!(1),
            serde_json::json!([true]),
            serde_json::Value::Null,
        ] {
            assert!(!parse_boolean_frontmatter(Some(&value)));
        }
        assert!(!parse_boolean_frontmatter(None));
    }

    use super::*;

    #[test]
    fn parse_frontmatter_uses_bun_duplicate_mapping_values() {
        let parsed = parse_frontmatter(
            "---\ndescription: first\ndescription: last\nhooks: &x {a: 1, a: 2}\ncopy: *x\n---\nPrompt",
        );
        assert_eq!(
            parsed.frontmatter.get("description"),
            Some(&serde_json::json!("last"))
        );
        assert_eq!(
            parsed.frontmatter.get("hooks"),
            Some(&serde_json::json!({"a": 2}))
        );
        assert_eq!(
            parsed.frontmatter.get("hooks"),
            parsed.frontmatter.get("copy")
        );
        assert_eq!(parsed.content, "Prompt");
    }

    #[test]
    fn parse_frontmatter_extracts_yaml_and_content_like_official() {
        let parsed = parse_frontmatter(
            "---\nname: Custom\ndescription: 42\nkeep-coding-instructions: true\n---\n# Body\nPrompt",
        );
        assert_eq!(
            parsed.frontmatter.get("name").and_then(Value::as_str),
            Some("Custom")
        );
        assert_eq!(
            coerce_description_to_string(parsed.frontmatter.get("description")).as_deref(),
            Some("42")
        );
        assert_eq!(parsed.content, "# Body\nPrompt");
    }

    #[test]
    fn positive_int_frontmatter_matches_number_and_parse_int_inputs() {
        assert_eq!(
            parse_positive_int_from_frontmatter(Some(&Value::from(4))),
            Some(4)
        );
        assert_eq!(
            parse_positive_int_from_frontmatter(Some(&Value::String("  +12turns".to_string()))),
            Some(12)
        );
        assert_eq!(
            parse_positive_int_from_frontmatter(Some(&Value::String("0".to_string()))),
            None
        );
        assert_eq!(
            parse_positive_int_from_frontmatter(Some(&Value::String("nope".to_string()))),
            None
        );
    }

    #[test]
    fn parse_frontmatter_retries_with_quoted_problematic_scalars_like_official() {
        let parsed = parse_frontmatter(
            "---   \ndescription: Explains APIs: robustly\npaths: **/*.{ts,tsx}\n---\nPrompt",
        );
        assert_eq!(
            parsed
                .frontmatter
                .get("description")
                .and_then(Value::as_str),
            Some("Explains APIs: robustly")
        );
        assert_eq!(
            parsed.frontmatter.get("paths").and_then(Value::as_str),
            Some("**/*.{ts,tsx}")
        );
        assert_eq!(parsed.content, "Prompt");
    }

    #[test]
    fn split_path_in_frontmatter_expands_braces_like_official() {
        // The three doc-comment examples at CC frontmatterParser.ts:184-187.
        assert_eq!(
            split_path_in_frontmatter(&Value::String("a, b".to_string())),
            vec!["a", "b"]
        );
        assert_eq!(
            split_path_in_frontmatter(&Value::String("a, src/*.{ts,tsx}".to_string())),
            vec!["a", "src/*.ts", "src/*.tsx"]
        );
        assert_eq!(
            split_path_in_frontmatter(&Value::String("{a,b}/{c,d}".to_string())),
            vec!["a/c", "a/d", "b/c", "b/d"]
        );
        assert_eq!(
            split_path_in_frontmatter(&serde_json::json!(["a", "src/*.{ts,tsx}"])),
            vec!["a", "src/*.ts", "src/*.tsx"]
        );
    }

    #[test]
    fn split_path_in_frontmatter_matches_official_regex_edge_cases() {
        // `[^}]+` requires a non-empty alternative list, so `{}` is left alone.
        assert_eq!(
            split_path_in_frontmatter(&Value::String("a{}b".to_string())),
            vec!["a{}b"]
        );
        // No closing brace: the regex does not match, pattern passes through.
        assert_eq!(
            split_path_in_frontmatter(&Value::String("src/{ts".to_string())),
            vec!["src/{ts"]
        );
        // Alternatives are trimmed (CC `alt.trim()`).
        assert_eq!(
            split_path_in_frontmatter(&Value::String("x.{ts, tsx }".to_string())),
            vec!["x.ts", "x.tsx"]
        );
        // Nested braces are NOT handled as nesting. `[^}]+` stops at the first
        // `}`, so the alternative list is `a,{b,c` and the trailing `}` becomes
        // the suffix appended to every alternative — only the one that happens
        // to hold the inner `{` re-expands. Verified against CC's own regex:
        //   node -e "…expandBraces('{a,{b,c}}')" -> ["a}","b","c}"]
        assert_eq!(
            split_path_in_frontmatter(&Value::String("{a,{b,c}}".to_string())),
            vec!["a}", "b", "c}"]
        );
        // Signed brace depth: the stray `}` drops depth to -1, so the following
        // comma is NOT a separator.
        assert_eq!(
            split_path_in_frontmatter(&Value::String("a},b".to_string())),
            vec!["a},b"]
        );
        // Non-string, non-array frontmatter yields nothing.
        assert!(split_path_in_frontmatter(&Value::Bool(true)).is_empty());
        assert!(split_path_in_frontmatter(&Value::Null).is_empty());
    }

    #[test]
    fn parse_shell_frontmatter_accepts_official_shell_values() {
        assert_eq!(
            parse_shell_frontmatter(Some(&Value::String("bash".to_string())), "skill"),
            Some(FrontmatterShell::Bash)
        );
        assert_eq!(
            parse_shell_frontmatter(Some(&Value::String("PowerShell".to_string())), "skill"),
            Some(FrontmatterShell::PowerShell)
        );
        assert_eq!(
            parse_shell_frontmatter(Some(&Value::String("fish".to_string())), "skill"),
            None
        );
        assert_eq!(parse_shell_frontmatter(None, "skill"), None);
    }
}

#[cfg(test)]
mod plugin_frontmatter_parity_tests {
    use super::*;
    /// Actual Bun source parseFrontmatter oracle, including delimiter and ECMAScript whitespace.
    #[test]
    fn plugin_frontmatter_extraction_matches_official_bun() {
        let cases: Value = serde_json::from_str(r###"[{"input":"---\ndescription: hi---\nBody","result":{"frontmatter":{"description":"hi"},"content":"Body"}},{"input":" ---\ndescription: hi\n---\nBody","result":{"frontmatter":{},"content":" ---\ndescription: hi\n---\nBody"}},{"input":"---\ndescription: hi\n---   \n\nBody","result":{"frontmatter":{"description":"hi"},"content":"Body"}},{"input":"---\ndescription: hi\n---Tail","result":{"frontmatter":{"description":"hi"},"content":"Tail"}},{"input":"---\r\ndescription: hi\r\n---\r\nBody","result":{"frontmatter":{"description":"hi"},"content":"Body"}},{"input":"---\ndescription: hi\n---\nBody","result":{"frontmatter":{},"content":"---\ndescription: hi\n---\nBody"}},{"input":"---﻿\ndescription: hi\n---\nBody","result":{"frontmatter":{"description":"hi"},"content":"Body"}},{"input":"---\ndescription: first\ndescription: last\n---\nBody","result":{"frontmatter":{"description":"last"},"content":"Body"}}]"###).unwrap();
        for case in cases.as_array().unwrap() {
            let parsed = parse_frontmatter(case["input"].as_str().unwrap());
            assert_eq!(
                serde_json::json!({"frontmatter": parsed.frontmatter, "content": parsed.content}),
                case["result"],
                "{}",
                case["input"]
            );
        }
    }
}
