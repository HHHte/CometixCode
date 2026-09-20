//! Bash parser API consumed by security AST analysis.
//!
//! Maps to: CC `utils/bash/parser.ts` `PARSE_ABORTED`,
//! `parseCommandRaw(...)`, and the 10,000 UTF-16-unit command limit.

use tree_sitter::Tree;

const MAX_COMMAND_LENGTH: usize = 10_000;

#[derive(Debug)]
pub enum ParseCommandRawResult {
    Parsed(Tree),
    Unavailable,
    ParseAborted,
}

/// Maps to CC `parseCommandRaw(command)`.
pub fn parse_command_raw(command: &str) -> ParseCommandRawResult {
    if command.is_empty() || command.encode_utf16().count() > MAX_COMMAND_LENGTH {
        return ParseCommandRawResult::Unavailable;
    }
    match super::bash_parser::parse(command) {
        Ok(Some(tree)) => ParseCommandRawResult::Parsed(tree),
        Ok(None) => ParseCommandRawResult::ParseAborted,
        Err(_) => ParseCommandRawResult::Unavailable,
    }
}

/// Parser-level syntax check used by the legacy differential guard.
pub fn has_parse_error(command: &str) -> bool {
    match parse_command_raw(command) {
        ParseCommandRawResult::Parsed(tree) => tree.root_node().has_error(),
        ParseCommandRawResult::Unavailable | ParseCommandRawResult::ParseAborted => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_parser_distinguishes_unavailable_length_and_syntax_tree() {
        assert!(matches!(
            parse_command_raw(""),
            ParseCommandRawResult::Unavailable
        ));
        assert!(matches!(
            parse_command_raw(&"x".repeat(MAX_COMMAND_LENGTH + 1)),
            ParseCommandRawResult::Unavailable
        ));
        assert!(matches!(
            parse_command_raw("printf ok"),
            ParseCommandRawResult::Parsed(_)
        ));
        assert!(has_parse_error("echo 'unterminated"));
    }
}
