//! Parser for simple `sed -i` edit commands.
//!
//! Maps to: CC `tools/BashTool/sedEditParser.ts`.
//! This module recognizes the same simple in-place substitution shape used by
//! Claude Code's permission UI and can apply the previewed substitution without
//! shelling out when `_simulatedSedEdit` is approved.

use regex::{Captures, RegexBuilder};

const BACKSLASH_PLACEHOLDER: &str = "\0BACKSLASH\0";
const PLUS_PLACEHOLDER: &str = "\0PLUS\0";
const QUESTION_PLACEHOLDER: &str = "\0QUESTION\0";
const PIPE_PLACEHOLDER: &str = "\0PIPE\0";
const LPAREN_PLACEHOLDER: &str = "\0LPAREN\0";
const RPAREN_PLACEHOLDER: &str = "\0RPAREN\0";

/// Maps to: CC `SedEditInfo`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SedEditInfo {
    pub file_path: String,
    pub pattern: String,
    pub replacement: String,
    pub flags: String,
    pub extended_regex: bool,
}

/// Maps to: CC `isSedInPlaceEdit`.
pub fn is_sed_in_place_edit(command: &str) -> bool {
    parse_sed_edit_command(command).is_some()
}

/// Maps to: CC `parseSedEditCommand`.
pub fn parse_sed_edit_command(command: &str) -> Option<SedEditInfo> {
    let trimmed = command.trim();
    let without_sed = trimmed.strip_prefix("sed")?;
    if !without_sed
        .chars()
        .next()
        .is_some_and(|ch| ch.is_whitespace())
    {
        return None;
    }

    let args = parse_shell_words(without_sed.trim())?;
    let mut has_in_place_flag = false;
    let mut extended_regex = false;
    let mut expression: Option<String> = None;
    let mut file_path: Option<String> = None;
    let mut index = 0usize;

    while index < args.len() {
        let arg = &args[index];
        if arg == "-i" || arg == "--in-place" {
            has_in_place_flag = true;
            index += 1;
            if index < args.len() {
                let next = &args[index];
                if !next.starts_with('-') && (next.is_empty() || next.starts_with('.')) {
                    index += 1;
                }
            }
            continue;
        }
        if arg.starts_with("-i") {
            has_in_place_flag = true;
            index += 1;
            continue;
        }
        if arg == "-E" || arg == "-r" || arg == "--regexp-extended" {
            extended_regex = true;
            index += 1;
            continue;
        }
        if arg == "-e" || arg == "--expression" {
            if expression.is_some() || index + 1 >= args.len() {
                return None;
            }
            expression = Some(args[index + 1].clone());
            index += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--expression=") {
            if expression.is_some() {
                return None;
            }
            expression = Some(value.to_string());
            index += 1;
            continue;
        }
        if arg.starts_with('-') {
            return None;
        }

        if expression.is_none() {
            expression = Some(arg.clone());
        } else if file_path.is_none() {
            file_path = Some(arg.clone());
        } else {
            return None;
        }
        index += 1;
    }

    if !has_in_place_flag {
        return None;
    }
    let expression = expression?;
    let file_path = file_path?;
    if shell_arg_has_glob_pattern(&file_path) {
        return None;
    }
    parse_sed_substitution_expression(&expression, file_path, extended_regex)
}

fn parse_sed_substitution_expression(
    expression: &str,
    file_path: String,
    extended_regex: bool,
) -> Option<SedEditInfo> {
    let rest = expression.strip_prefix("s/")?;
    let mut pattern = String::new();
    let mut replacement = String::new();
    let mut flags = String::new();
    let mut state = SedParseState::Pattern;
    let mut chars = rest.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let next = chars.next()?;
            match state {
                SedParseState::Pattern => {
                    pattern.push(ch);
                    pattern.push(next);
                }
                SedParseState::Replacement => {
                    replacement.push(ch);
                    replacement.push(next);
                }
                SedParseState::Flags => {
                    flags.push(ch);
                    flags.push(next);
                }
            }
            continue;
        }

        if ch == '/' {
            match state {
                SedParseState::Pattern => state = SedParseState::Replacement,
                SedParseState::Replacement => state = SedParseState::Flags,
                SedParseState::Flags => return None,
            }
            continue;
        }

        match state {
            SedParseState::Pattern => pattern.push(ch),
            SedParseState::Replacement => replacement.push(ch),
            SedParseState::Flags => flags.push(ch),
        }
    }

    if state != SedParseState::Flags
        || !flags
            .chars()
            .all(|ch| matches!(ch, 'g' | 'p' | 'i' | 'm' | 'I' | 'M' | '1'..='9'))
    {
        return None;
    }

    Some(SedEditInfo {
        file_path,
        pattern,
        replacement,
        flags,
        extended_regex,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SedParseState {
    Pattern,
    Replacement,
    Flags,
}

/// Maps to: CC `applySedSubstitution`.
pub fn apply_sed_substitution(content: &str, sed_info: &SedEditInfo) -> String {
    let mut builder = RegexBuilder::new(&sed_pattern_to_rust_regex(sed_info));
    builder.case_insensitive(sed_info.flags.contains('i') || sed_info.flags.contains('I'));
    builder.multi_line(sed_info.flags.contains('m') || sed_info.flags.contains('M'));

    let Ok(regex) = builder.build() else {
        return content.to_string();
    };

    if sed_info.flags.contains('g') {
        regex
            .replace_all(content, |captures: &Captures<'_>| {
                sed_replacement_for_match(&sed_info.replacement, captures)
            })
            .into_owned()
    } else {
        regex
            .replace(content, |captures: &Captures<'_>| {
                sed_replacement_for_match(&sed_info.replacement, captures)
            })
            .into_owned()
    }
}

fn sed_pattern_to_rust_regex(sed_info: &SedEditInfo) -> String {
    let mut pattern = sed_info.pattern.replace("\\/", "/");

    // Maps to CC's BRE→ERE conversion: in basic regex mode, escaped `+`,
    // `?`, `|`, `(` and `)` are regex metacharacters, while unescaped ones are
    // literals. Rust regex syntax is ERE-like, so we perform the same sentinel
    // conversion as the TypeScript source.
    if !sed_info.extended_regex {
        pattern = pattern
            .replace("\\\\", BACKSLASH_PLACEHOLDER)
            .replace("\\+", PLUS_PLACEHOLDER)
            .replace("\\?", QUESTION_PLACEHOLDER)
            .replace("\\|", PIPE_PLACEHOLDER)
            .replace("\\(", LPAREN_PLACEHOLDER)
            .replace("\\)", RPAREN_PLACEHOLDER)
            .replace('+', "\\+")
            .replace('?', "\\?")
            .replace('|', "\\|")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace(BACKSLASH_PLACEHOLDER, "\\\\")
            .replace(PLUS_PLACEHOLDER, "+")
            .replace(QUESTION_PLACEHOLDER, "?")
            .replace(PIPE_PLACEHOLDER, "|")
            .replace(LPAREN_PLACEHOLDER, "(")
            .replace(RPAREN_PLACEHOLDER, ")");
    }

    pattern
}

fn sed_replacement_for_match(replacement: &str, captures: &Captures<'_>) -> String {
    let whole_match = captures.get(0).map(|m| m.as_str()).unwrap_or_default();
    let mut out = String::new();
    let mut chars = replacement.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    '/' => out.push('/'),
                    '&' => out.push('&'),
                    _ => {
                        out.push('\\');
                        out.push(next);
                    }
                }
            } else {
                out.push(ch);
            }
        } else if ch == '&' {
            out.push_str(whole_match);
        } else {
            out.push(ch);
        }
    }

    out
}

fn shell_arg_has_glob_pattern(value: &str) -> bool {
    value.chars().any(|ch| matches!(ch, '*' | '?' | '[' | ']'))
}

fn parse_shell_words(input: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut quote: Option<char> = None;
    let mut in_word = false;

    while let Some(ch) = chars.next() {
        match quote {
            Some('\'') => {
                if ch == '\'' {
                    quote = None;
                } else {
                    current.push(ch);
                }
            }
            Some('"') => {
                if ch == '"' {
                    quote = None;
                } else if ch == '\\' {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                } else {
                    current.push(ch);
                }
            }
            _ if ch == '\'' || ch == '"' => {
                quote = Some(ch);
                in_word = true;
            }
            _ if ch.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut current));
                    in_word = false;
                }
            }
            _ if matches!(ch, '|' | '&' | ';' | '<' | '>' | '(' | ')') => return None,
            _ if ch == '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                    in_word = true;
                }
            }
            _ => {
                current.push(ch);
                in_word = true;
            }
        }
    }

    if quote.is_some() {
        return None;
    }
    if in_word {
        words.push(current);
    }
    Some(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sed_edit_command_extracts_official_info() {
        let info = parse_sed_edit_command(" sed -E -i.bak 's/foo+/bar/g' src/main.rs ")
            .expect("valid sed edit");
        assert_eq!(info.file_path, "src/main.rs");
        assert_eq!(info.pattern, "foo+");
        assert_eq!(info.replacement, "bar");
        assert_eq!(info.flags, "g");
        assert!(info.extended_regex);
        assert!(is_sed_in_place_edit("sed -i '' 's/old/new/' file.txt"));
    }

    #[test]
    fn parse_sed_edit_command_rejects_multi_file_or_glob_shapes() {
        assert!(parse_sed_edit_command("sed 's/a/b/' file.txt").is_none());
        assert!(parse_sed_edit_command("sed -i 's/a/b/' *.txt").is_none());
        assert!(parse_sed_edit_command("sed -i 's/a/b/' a.txt b.txt").is_none());
        assert!(parse_sed_edit_command("sed -n -i 's/a/b/' file.txt").is_none());
    }

    #[test]
    fn apply_sed_substitution_matches_official_preview_rules() {
        let info = parse_sed_edit_command("sed -i 's/foo/bar/g' file.txt").unwrap();
        assert_eq!(
            apply_sed_substitution("foo foo\nnope", &info),
            "bar bar\nnope"
        );

        let bre_info = parse_sed_edit_command("sed -i 's/fo\\+/BAR/' file.txt").unwrap();
        assert_eq!(apply_sed_substitution("foooo foo", &bre_info), "BAR foo");

        let amp_info = parse_sed_edit_command("sed -i 's/foo/[&] \\&/' file.txt").unwrap();
        assert_eq!(apply_sed_substitution("foo", &amp_info), "[foo] &");
    }
}
