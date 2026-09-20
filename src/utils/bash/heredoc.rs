//! Heredoc extraction used by the legacy Bash security path.
//!
//! Maps to: CC `utils/bash/heredoc.ts` `extractHeredocs(..., { quotedOnly: true })`.
//! This owner deliberately strips only statically-delimited quoted/escaped
//! heredocs. Any ambiguous form is left visible to downstream validators.

use regex::Regex;
use std::sync::LazyLock;

static HEREDOC_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"<<(-)?[ \t]*(?:'((?:\\)?[A-Za-z0-9_]+)'|\"((?:\\)?[A-Za-z0-9_]+)\"|\\([A-Za-z0-9_]+)|([A-Za-z0-9_]+))"#,
    )
    .expect("valid heredoc start regex")
});

fn is_unquoted_position(command: &str, target: usize) -> bool {
    let mut single = false;
    let mut double = false;
    let mut comment = false;
    let mut escaped = false;
    for character in command[..target].chars() {
        if character == '\n' {
            comment = false;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && !single {
            escaped = true;
            continue;
        }
        if character == '\'' && !double {
            single = !single;
            continue;
        }
        if character == '"' && !single {
            double = !double;
            continue;
        }
        if character == '#' && !single && !double {
            comment = true;
        }
    }
    !single && !double && !comment
}

fn closing_delimiter_range(
    command: &str,
    operator_end: usize,
    delimiter: &str,
    strip_tabs: bool,
) -> Option<(usize, usize)> {
    let line_end = command[operator_end..].find('\n')? + operator_end;
    let mut cursor = line_end + 1;
    while cursor <= command.len() {
        let end = command[cursor..]
            .find('\n')
            .map(|offset| cursor + offset)
            .unwrap_or(command.len());
        let line = &command[cursor..end];
        let comparable = if strip_tabs {
            line.trim_start_matches('\t')
        } else {
            line
        };
        if comparable == delimiter {
            return Some((line_end, end));
        }
        if end == command.len() {
            break;
        }
        cursor = end + 1;
    }
    None
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeredocInfo {
    pub full_text: String,
    pub delimiter: String,
    pub operator_start_index: usize,
    pub operator_end_index: usize,
    pub content_start_index: usize,
    pub content_end_index: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeredocExtractionResult {
    pub processed_command: String,
    pub heredocs: std::collections::BTreeMap<String, HeredocInfo>,
}

/// Maps to CC `containsHeredoc(command)`.
pub fn contains_heredoc(command: &str) -> bool {
    HEREDOC_START.is_match(command)
}

/// Maps to CC `extractHeredocs(command, options)`.
pub fn extract_heredocs(command: &str, quoted_only: bool) -> HeredocExtractionResult {
    let unchanged = || HeredocExtractionResult {
        processed_command: command.to_string(),
        heredocs: std::collections::BTreeMap::new(),
    };
    if !command.contains("<<") || command.contains("$'") || command.contains("$\"") {
        return unchanged();
    }
    let first = command.find("<<").unwrap_or_default();
    if command[..first].contains('`') {
        return unchanged();
    }
    let before_first = &command[..first];
    if before_first.matches("((").count() > before_first.matches("))").count() {
        return unchanged();
    }

    let mut matches = Vec::new();
    let mut search = 0usize;
    while let Some(captures) = HEREDOC_START.captures(&command[search..]) {
        let matched = captures.get(0).expect("whole heredoc match");
        let start = search + matched.start();
        let end = search + matched.end();
        search = end;
        if command[..start].ends_with('<')
            || command[end..].starts_with('<')
            || !is_unquoted_position(command, start)
        {
            continue;
        }
        let quoted_delimiter = captures
            .get(2)
            .or_else(|| captures.get(3))
            .or_else(|| captures.get(4));
        if quoted_only && quoted_delimiter.is_none() {
            // The source tracks skipped unquoted ranges to prevent nested
            // extraction. Returning unchanged is the same fail-closed direction.
            return unchanged();
        }
        let Some(delimiter_match) = quoted_delimiter.or_else(|| captures.get(5)) else {
            continue;
        };
        let delimiter = delimiter_match.as_str();
        let Some((content_start, content_end)) =
            closing_delimiter_range(command, end, delimiter, captures.get(1).is_some())
        else {
            continue;
        };
        let full_text = format!(
            "{}{}",
            &command[start..end],
            &command[content_start..content_end]
        );
        matches.push(HeredocInfo {
            full_text,
            delimiter: delimiter.to_string(),
            operator_start_index: start,
            operator_end_index: end,
            content_start_index: content_start,
            content_end_index: content_end,
        });
        search = content_end;
    }
    if matches.is_empty() {
        return unchanged();
    }
    let content_ranges = matches
        .iter()
        .map(|info| {
            (
                info.operator_start_index,
                info.content_start_index,
                info.content_end_index,
            )
        })
        .collect::<Vec<_>>();
    matches.retain(|candidate| {
        !content_ranges.iter().any(|(operator_start, start, end)| {
            candidate.operator_start_index != *operator_start
                && candidate.operator_start_index > *start
                && candidate.operator_start_index < *end
        })
    });
    let distinct_content_starts = matches
        .iter()
        .map(|info| info.content_start_index)
        .collect::<std::collections::BTreeSet<_>>();
    if matches.is_empty() || distinct_content_starts.len() != matches.len() {
        return unchanged();
    }

    matches.sort_by_key(|info| std::cmp::Reverse(info.content_end_index));
    let salt = uuid::Uuid::new_v4().simple().to_string()[..16].to_string();
    let mut processed_command = command.to_string();
    let mut heredocs = std::collections::BTreeMap::new();
    let total = matches.len();
    for (index, info) in matches.into_iter().enumerate() {
        let placeholder = format!("__HEREDOC_{}_{}__", total - 1 - index, salt);
        processed_command = format!(
            "{}{}{}{}",
            &processed_command[..info.operator_start_index],
            placeholder,
            &processed_command[info.operator_end_index..info.content_start_index],
            &processed_command[info.content_end_index..]
        );
        heredocs.insert(placeholder, info);
    }
    HeredocExtractionResult {
        processed_command,
        heredocs,
    }
}

/// Maps to CC `restoreHeredocs(parts, heredocs)`.
pub fn restore_heredocs(
    parts: Vec<String>,
    heredocs: &std::collections::BTreeMap<String, HeredocInfo>,
) -> Vec<String> {
    if heredocs.is_empty() {
        return parts;
    }
    parts
        .into_iter()
        .map(|mut part| {
            for (placeholder, info) in heredocs {
                part = part.replace(placeholder, &info.full_text);
            }
            part
        })
        .collect()
}

/// Replace safely quoted heredocs with inert placeholders. Unquoted heredocs
/// remain intact because their bodies perform command/parameter expansion.
pub fn strip_quoted_heredoc_bodies(command: &str) -> String {
    if !command.contains("<<") || command.contains("$'") || command.contains("$\"") {
        return command.to_string();
    }
    let first = command.find("<<").unwrap_or_default();
    if command[..first].contains('`') {
        return command.to_string();
    }
    let before_first = &command[..first];
    if before_first.matches("((").count() > before_first.matches("))").count() {
        return command.to_string();
    }

    let mut output = String::with_capacity(command.len());
    let mut cursor = 0usize;
    let mut search = 0usize;
    let mut index = 0usize;
    while let Some(captures) = HEREDOC_START.captures(&command[search..]) {
        let matched = captures.get(0).expect("whole heredoc match");
        let start = search + matched.start();
        let end = search + matched.end();
        search = end;

        if command[..start].ends_with('<')
            || command[end..].starts_with('<')
            || !is_unquoted_position(command, start)
        {
            continue;
        }
        let quoted_delimiter = captures
            .get(2)
            .or_else(|| captures.get(3))
            .or_else(|| captures.get(4));
        if quoted_delimiter.is_none() {
            // Conservatively retain the complete command when an unquoted
            // heredoc is present; a quoted-looking marker inside its body is
            // not a nested shell operator.
            return command.to_string();
        }
        let delimiter = quoted_delimiter.expect("checked delimiter").as_str();
        let Some((operator_line_end, content_end)) =
            closing_delimiter_range(command, end, delimiter, captures.get(1).is_some())
        else {
            continue;
        };
        if start < cursor {
            continue;
        }
        output.push_str(&command[cursor..start]);
        output.push_str(&format!("__HEREDOC_{index}__"));
        output.push_str(&command[end..operator_line_end]);
        index += 1;
        cursor = content_end;
        search = content_end;
    }
    if index == 0 {
        return command.to_string();
    }
    output.push_str(&command[cursor..]);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_bodies_are_hidden_but_unquoted_bodies_remain_visible() {
        assert_eq!(
            strip_quoted_heredoc_bodies("cat <<'EOF'\n$(id)\nEOF"),
            "cat __HEREDOC_0__"
        );
        assert_eq!(
            strip_quoted_heredoc_bodies("cat <<EOF\n$(id)\nEOF"),
            "cat <<EOF\n$(id)\nEOF"
        );
        assert_eq!(
            strip_quoted_heredoc_bodies("cat <<-'EOF'\n\tbody\n\tEOF\necho done"),
            "cat __HEREDOC_0__\necho done"
        );
        assert_eq!(
            strip_quoted_heredoc_bodies("cat <<'EOF'; echo same\nbody\nEOF"),
            "cat __HEREDOC_0__; echo same"
        );
    }

    #[test]
    fn extract_and_restore_heredocs_match_source_shaped_placeholders() {
        let command = "cat <<'EOF' && echo done\nbody; $(literal)\nEOF\nprintf tail";
        let extraction = extract_heredocs(command, false);
        assert_eq!(extraction.heredocs.len(), 1);
        assert!(extraction.processed_command.contains("__HEREDOC_0_"));
        assert!(extraction.processed_command.contains(" && echo done"));
        assert!(!extraction.processed_command.contains("body; $(literal)"));
        let placeholder = extraction.heredocs.keys().next().unwrap();
        assert_eq!(
            restore_heredocs(vec![format!("cat {placeholder}")], &extraction.heredocs),
            ["cat <<'EOF'\nbody; $(literal)\nEOF"]
        );
        assert!(contains_heredoc(command));
    }

    #[test]
    fn quoted_marker_inside_unquoted_body_is_not_extracted() {
        let command = "cat <<EOF\n<<'SAFE'\n$(id)\nSAFE\nEOF";
        assert_eq!(strip_quoted_heredoc_bodies(command), command);
    }
}
