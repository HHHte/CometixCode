//! Maps to: CC `utils/hyperlink.ts`.
//!
//! OSC 8 hyperlink construction. CC exports this from one module and every
//! consumer imports it (`components/shell/OutputLine.tsx:5`,
//! `tools/MCPTool/UI.tsx:17`, `components/Markdown.tsx`); the Rust port had
//! byte-identical private copies in `components/markdown.rs` and
//! `tools/mcp_tool/ui.rs` instead. This module is the CC-named owner.

use iocraft::prelude::supports_hyperlinks;

/// Maps to: CC `utils/hyperlink.ts:7` `OSC8_START`.
pub const OSC8_START: &str = "\x1b]8;;";
/// Maps to: CC `utils/hyperlink.ts:8` `OSC8_END` — BEL, "more widely
/// supported" than ST.
pub const OSC8_END: &str = "\x07";

/// Maps to: CC `utils/hyperlink.ts:25-42` `createHyperlink`.
///
/// `content` is CC's optional display text: shown only when the terminal
/// supports hyperlinks, ignored on the fallback (which emits the bare URL).
/// The `chalk.blue(displayText)` wrapper is the basic ANSI blue pair
/// (`\x1b[34m` … `\x1b[39m`) CC deliberately uses instead of a theme RGB —
/// wrap-ansi preserves the basic pair across line breaks, RGB with OSC 8 it
/// does not.
pub fn create_hyperlink(url: &str, content: Option<&str>) -> String {
    create_hyperlink_with_support(url, content, supports_hyperlinks())
}

/// Maps to: CC `createHyperlink`'s `options?.supportsHyperlinks` override —
/// the escape hatch its own tests use, so terminal capability cannot decide a
/// test outcome.
pub fn create_hyperlink_with_support(
    url: &str,
    content: Option<&str>,
    has_support: bool,
) -> String {
    if !has_support {
        return url.to_string();
    }

    let display_text = content.unwrap_or(url);
    format!("{OSC8_START}{url}{OSC8_END}\x1b[34m{display_text}\x1b[39m{OSC8_START}{OSC8_END}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyperlink_matches_official_osc8_and_chalk_blue_bytes() {
        assert_eq!(
            create_hyperlink_with_support("https://example.test", None, true),
            "\x1b]8;;https://example.test\x07\x1b[34mhttps://example.test\x1b[39m\x1b]8;;\x07"
        );
        assert_eq!(
            create_hyperlink_with_support("https://example.test", Some("#ops"), true),
            "\x1b]8;;https://example.test\x07\x1b[34m#ops\x1b[39m\x1b]8;;\x07"
        );
        // CC :32-34 — no support means the bare URL, and `content` is dropped.
        assert_eq!(
            create_hyperlink_with_support("https://example.test", Some("#ops"), false),
            "https://example.test"
        );
    }
}
