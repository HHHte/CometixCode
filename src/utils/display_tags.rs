//! Maps to: CC `utils/displayTags.ts`.

/// Maps to CC `utils/displayTags.ts:15-28` `stripDisplayTags`.
pub fn strip_display_tags(text: &str) -> String {
    let result = strip_display_tags_allow_empty(text);
    if result.is_empty() {
        text.to_string()
    } else {
        result
    }
}

/// Maps to CC `utils/displayTags.ts:36-38` `stripDisplayTagsAllowEmpty`.
pub fn strip_display_tags_allow_empty(text: &str) -> String {
    static PATTERN: std::sync::LazyLock<regress::Regex> = std::sync::LazyLock::new(|| {
        regress::Regex::new(r"<([a-z][\w-]*)(?:\s[^>]*)?>[\s\S]*?</\1>\n?")
            .expect("constant display-tag pattern")
    });
    let mut result = String::new();
    let mut end = 0;
    for matched in PATTERN.find_iter(text) {
        result.push_str(&text[end..matched.range.start]);
        end = matched.range.end;
    }
    result.push_str(&text[end..]);
    result.trim().to_string()
}

/// Maps to: CC `utils/displayTags.ts:41-50` `stripIdeContextTags`.
/// The two alternatives retain the source backreference without requiring a
/// backtracking regex engine. User-authored HTML must survive resubmission.
pub fn strip_ide_context_tags(text: &str) -> String {
    static PATTERN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"(?s)<ide_opened_file(?:\s[^>]*)?>.*?</ide_opened_file>\n?|<ide_selection(?:\s[^>]*)?>.*?</ide_selection>\n?",
        )
        .expect("constant IDE context pattern")
    });
    PATTERN.replace_all(text, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ide_context_tags_matches_official_resubmit_filter() {
        assert_eq!(
            strip_ide_context_tags(
                "<ide_selection path=\"a\">one\ntwo</ide_selection>\n<code>keep</code>"
            ),
            "<code>keep</code>"
        );
        assert_eq!(
            strip_ide_context_tags("<ide_opened_file>x</ide_selection>"),
            "<ide_opened_file>x</ide_selection>"
        );
        assert_eq!(
            strip_ide_context_tags("<ide_opened_file>x</ide_opened_file>\n"),
            ""
        );
    }
    #[test]
    fn strip_display_tags_matches_official_backreference_and_fallback() {
        assert_eq!(
            strip_display_tags("<fooBar>internal</fooBar>\nvisible"),
            "visible"
        );
        assert_eq!(
            strip_display_tags("<foo a=\"x\">hidden</foo> visible"),
            "visible"
        );
        assert_eq!(strip_display_tags("<foo>only</foo>"), "<foo>only</foo>");
        assert_eq!(strip_display_tags_allow_empty("<foo>only</foo>"), "");
        assert_eq!(
            strip_display_tags("<foo>keep</bar> tail"),
            "<foo>keep</bar> tail"
        );
        assert_eq!(
            strip_display_tags("<Foo>keep</Foo> tail"),
            "<Foo>keep</Foo> tail"
        );
    }
}
