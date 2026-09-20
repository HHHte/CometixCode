//! Maps to: CC `utils/slashCommandParsing.ts`.

#[derive(Debug, Clone)]
pub struct ParsedSlashCommand {
    pub name: String,
    pub args: String,
}

/// Maps to CC `utils/slashCommandParsing.ts:25-63#parseSlashCommand`.
/// The established native name/args carrier preserves registry consumers;
/// the `(MCP)` marker is retained in name exactly as the source parser does.
pub fn parse_slash_command(input: &str) -> Option<ParsedSlashCommand> {
    // ECMAScript String.trim: U+FEFF is whitespace, U+0085 is not.
    let trimmed = input.trim_matches(|character| {
        matches!(character,
            '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' |
            '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
            '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    });
    let without_slash = trimmed.strip_prefix('/')?;
    let words: Vec<_> = without_slash.split(' ').collect();
    let first = words.first().filter(|word| !word.is_empty())?;
    let is_mcp = words.get(1) == Some(&"(MCP)");
    let name = if is_mcp {
        format!("{first} (MCP)")
    } else {
        first.to_string()
    };
    let args_start_index = if is_mcp { 2 } else { 1 };
    Some(ParsedSlashCommand {
        name,
        args: words[args_start_index..].join(" "),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_preserves_dynamic_mcp_command_case() {
        let parsed = parse_slash_command("/mcp__My_Server__daily-report eng roadmap").unwrap();
        assert_eq!(parsed.name, "mcp__My_Server__daily-report");
        assert_eq!(parsed.args, "eng roadmap");
    }
    #[test]
    fn parse_matches_official_whitespace_mcp_marker_and_argument_spacing() {
        for input in ["", " / ", "/ args", "plain text", "\u{85}/color green"] {
            assert!(parse_slash_command(input).is_none(), "{input:?}");
        }
        for (input, name, args) in [
            ("\u{feff}/color \u{85}green\u{feff}", "color", "\u{85}green"),
            (" /review   42 ", "review", "  42"),
            ("/mcp:tool (MCP)  arg1 arg2", "mcp:tool (MCP)", " arg1 arg2"),
            ("/mcp:tool  (MCP) value", "mcp:tool", " (MCP) value"),
            ("/color green\u{85}", "color", "green\u{85}"),
            ("/color\tgreen", "color\tgreen", ""),
        ] {
            let parsed = parse_slash_command(input).unwrap();
            assert_eq!(
                (parsed.name.as_str(), parsed.args.as_str()),
                (name, args),
                "{input:?}"
            );
        }
    }
}
