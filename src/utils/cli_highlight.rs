//! Official-shaped seam for `utils/cliHighlight.ts`.
//!
//! Claude Code loads `cli-highlight`/`highlight.js` asynchronously and passes a
//! `CliHighlight` object into markdown formatting. Rust keeps the same API
//! shape for markdown code fences and backs it with `syntect` plus `bat`'s
//! bundled syntax/theme assets. This is not a highlight.js grammar port, but it
//! owns the same markdown highlighter responsibilities: language support checks,
//! ANSI output for supported fences, and plaintext fallback for unsupported
//! fences.

use bat::assets::HighlightingAssets;
use std::{borrow::Cow, path::Path, sync::OnceLock};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style as SyntectStyle, Theme as SyntectTheme},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

const DEFAULT_MARKDOWN_SYNTAX_THEME: &str = "Monokai Extended";
const ANSI_RESET_FG: &str = "\x1b[39m";

#[derive(Clone, Copy, Debug, Default)]
pub struct CliHighlight;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HighlightOptions<'a> {
    pub language: &'a str,
}

pub fn get_cli_highlight() -> Option<CliHighlight> {
    Some(CliHighlight)
}

impl CliHighlight {
    pub fn supports_language(self, language: &str) -> bool {
        language_to_syntax(language).is_some()
    }

    pub fn highlight(self, code: &str, options: HighlightOptions<'_>) -> String {
        if code.is_empty() || code.contains('\x1b') || code.contains('\x07') {
            return code.to_string();
        }

        let Some(syntax) = language_to_syntax(options.language) else {
            return code.to_string();
        };
        if syntax.name == "Plain Text" {
            return code.to_string();
        }

        highlight_with_syntect(code, syntax).unwrap_or_else(|| code.to_string())
    }
}

pub fn highlight_markdown_code(text: &str, lang: Option<&str>) -> String {
    let Some(highlight) = get_cli_highlight() else {
        return text.to_string();
    };

    let language = lang
        .and_then(|value| value.split_whitespace().next())
        .filter(|value| highlight.supports_language(value))
        .unwrap_or("plaintext");

    highlight.highlight(text, HighlightOptions { language })
}

/// Mirrors official `getLanguageName(file_path)` with the current syntect
/// backend. It is read-only and returns `unknown` when no extension maps to a
/// bundled syntax.
#[cfg_attr(not(test), allow(dead_code))]
pub fn get_language_name(file_path: &str) -> String {
    let Some(extension) = Path::new(file_path)
        .extension()
        .and_then(|value| value.to_str())
    else {
        return "unknown".to_string();
    };
    language_to_syntax(extension)
        .filter(|syntax| syntax.name != "Plain Text")
        .map(|syntax| syntax.name.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

fn highlight_with_syntect(code: &str, syntax: &'static SyntaxReference) -> Option<String> {
    let syntax_set = syntax_set();
    let mut highlighter = HighlightLines::new(syntax, markdown_syntax_theme());
    let mut highlighted = String::new();
    let mut emitted_style = false;

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter.highlight_line(line, syntax_set).ok()?;
        for (style, text) in ranges {
            if text.is_empty() {
                continue;
            }
            if let Some(sequence) = foreground_sequence(style) {
                highlighted.push_str(&sequence);
                highlighted.push_str(text);
                highlighted.push_str(ANSI_RESET_FG);
                emitted_style = true;
            } else {
                highlighted.push_str(text);
            }
        }
    }

    if emitted_style {
        Some(highlighted)
    } else {
        None
    }
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(|| {
        let assets = HighlightingAssets::from_binary();
        assets
            .get_syntax_set()
            .cloned()
            .unwrap_or_else(|_| SyntaxSet::load_defaults_newlines())
    })
}

fn markdown_syntax_theme() -> &'static SyntectTheme {
    static THEME: OnceLock<SyntectTheme> = OnceLock::new();
    THEME.get_or_init(|| {
        HighlightingAssets::from_binary()
            .get_theme(DEFAULT_MARKDOWN_SYNTAX_THEME)
            .clone()
    })
}

fn language_to_syntax(language: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = syntax_set();
    let normalized = normalize_language(language);
    if is_plaintext_language(&normalized) {
        return Some(syntax_set.find_syntax_plain_text());
    }

    syntax_set
        .find_syntax_by_token(&normalized)
        .or_else(|| syntax_set.find_syntax_by_name(&normalized))
}

fn normalize_language(language: &str) -> Cow<'_, str> {
    let value = language.trim().trim_start_matches("language-");
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "plaintext" | "plain-text" | "plain" | "text" | "txt" => Cow::Borrowed("plaintext"),
        "shell" | "shell-script" | "sh" | "zsh" | "ksh" => Cow::Borrowed("bash"),
        "javascriptreact" | "react-js" => Cow::Borrowed("jsx"),
        "typescriptreact" | "react-ts" => Cow::Borrowed("tsx"),
        "c++" => Cow::Borrowed("cpp"),
        "c#" | "csharp" => Cow::Borrowed("cs"),
        "objective-c" | "objectivec" => Cow::Borrowed("m"),
        "golang" => Cow::Borrowed("go"),
        "py" => Cow::Borrowed("python"),
        "rb" => Cow::Borrowed("ruby"),
        "rs" => Cow::Borrowed("rust"),
        "yml" => Cow::Borrowed("yaml"),
        "md" => Cow::Borrowed("markdown"),
        "docker" => Cow::Borrowed("dockerfile"),
        _ => Cow::Owned(lower),
    }
}

fn is_plaintext_language(language: &str) -> bool {
    matches!(
        language,
        "plaintext" | "plain-text" | "plain" | "text" | "txt"
    )
}

fn foreground_sequence(style: SyntectStyle) -> Option<String> {
    let color = style.foreground;
    match color.a {
        // bat's ANSI theme encodes palette indexes as `r` with alpha 0.
        0 => Some(format!("\x1b[38;5;{}m", color.r)),
        // bat/syntect use alpha 1 as a terminal-default sentinel.
        1 => None,
        _ => Some(format!("\x1b[38;2;{};{};{}m", color.r, color.g, color.b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_highlight_matches_official_language_fallback_shape() {
        assert_eq!(
            highlight_markdown_code("fn main() {}", Some("unknown")),
            "fn main() {}"
        );
        assert_eq!(
            highlight_markdown_code("fn main() {}", None),
            "fn main() {}"
        );
        assert_eq!(
            highlight_markdown_code("fn main() {}", Some("plaintext")),
            "fn main() {}"
        );
    }

    #[test]
    fn cli_highlight_supports_syntect_language_aliases() {
        let highlight = get_cli_highlight().expect("cli highlighter should be available");
        assert!(highlight.supports_language("rust"));
        assert!(highlight.supports_language("rs"));
        assert!(highlight.supports_language("python"));
        assert!(highlight.supports_language("typescript"));
        assert!(highlight.supports_language("shell"));
        assert!(highlight.supports_language("plaintext"));
        assert!(!highlight.supports_language("definitely-not-a-language"));
    }

    #[test]
    fn cli_highlight_applies_syntect_ansi_for_supported_language() {
        let code = "fn main() { let x = 1; }";
        let highlighted = highlight_markdown_code(code, Some("rust"));
        assert!(highlighted.contains("\x1b[38;"));
        assert_eq!(strip_ansi_for_test(&highlighted), code);
    }

    #[test]
    fn cli_highlight_preserves_existing_ansi_input() {
        let code = "\x1b[31mred\x1b[0m";
        assert_eq!(highlight_markdown_code(code, Some("rust")), code);
    }

    #[test]
    fn cli_highlight_get_language_name_uses_syntect_extension_registry() {
        assert_eq!(get_language_name("src/main.rs"), "Rust");
        assert_eq!(get_language_name("README"), "unknown");
        assert_eq!(get_language_name("archive.unknownext"), "unknown");
    }

    fn strip_ansi_for_test(input: &str) -> String {
        let mut out = String::new();
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != '\x1b' {
                out.push(ch);
                continue;
            }
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
        }
        out
    }
}
