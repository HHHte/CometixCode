//! Bash command quoting helpers.
//!
//! Maps to: CC `utils/bash/shellQuoting.ts:1-113`.

use regex::Regex;
use std::sync::LazyLock;

/// Maps to CC `containsHeredoc(command)`.
fn contains_heredoc(command: &str) -> bool {
    static BIT_SHIFT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\d\s*<<\s*\d").expect("valid bit-shift regex"));
    static TEST_SHIFT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\[\[\s*\d+\s*<<\s*\d+\s*\]\]").expect("valid test shift regex")
    });
    static ARITHMETIC_SHIFT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\$\(\([^\n]*<<[^\n]*\)\)").expect("valid arithmetic regex"));
    static HEREDOC: LazyLock<Regex> = LazyLock::new(|| {
        // Rust regexes do not support backreferences. The alternatives preserve
        // the source matcher for bare, single/double-quoted, and escaped words.
        Regex::new(r#"<<-?\s*(?:[A-Za-z0-9_]+|'[A-Za-z0-9_]+'|"[A-Za-z0-9_]+"|\\[A-Za-z0-9_]+)"#)
            .expect("valid heredoc regex")
    });

    if BIT_SHIFT.is_match(command)
        || TEST_SHIFT.is_match(command)
        || ARITHMETIC_SHIFT.is_match(command)
    {
        return false;
    }
    HEREDOC.is_match(command)
}

/// Maps to CC `containsMultilineString(command)`.
fn contains_multiline_string(command: &str) -> bool {
    static SINGLE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"'(?:[^'\\]|\\.)*\n(?:[^'\\]|\\.)*'")
            .expect("valid single-quoted multiline regex")
    });
    static DOUBLE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"\"(?:[^\"\\]|\\.)*\n(?:[^\"\\]|\\.)*\""#)
            .expect("valid double-quoted multiline regex")
    });
    SINGLE.is_match(command) || DOUBLE.is_match(command)
}

fn single_quote_for_eval(command: &str) -> String {
    format!("'{}'", command.replace('\'', "'\"'\"'"))
}

/// Maps to CC `quoteShellCommand(command, addStdinRedirect)`.
pub fn quote_shell_command(command: &str, add_stdin_redirect: bool) -> String {
    if contains_heredoc(command) || contains_multiline_string(command) {
        let quoted = single_quote_for_eval(command);
        if contains_heredoc(command) || !add_stdin_redirect {
            return quoted;
        }
        return format!("{quoted} < /dev/null");
    }

    if add_stdin_redirect {
        crate::utils::bash::shell_quote::quote(&[command, "<", "/dev/null"])
    } else {
        crate::utils::bash::shell_quote::quote(&[command])
    }
}

/// Maps to CC `hasStdinRedirect(command)`.
pub fn has_stdin_redirect(command: &str) -> bool {
    let bytes = command.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] != b'<' {
            continue;
        }
        if index > 0
            && !bytes[index - 1].is_ascii_whitespace()
            && !b";&|".contains(&bytes[index - 1])
        {
            continue;
        }
        if bytes
            .get(index + 1)
            .is_some_and(|next| matches!(*next, b'<' | b'('))
        {
            continue;
        }
        let mut cursor = index + 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if cursor < bytes.len() {
            return true;
        }
    }
    false
}

/// Maps to CC `shouldAddStdinRedirect(command)`.
pub fn should_add_stdin_redirect(command: &str) -> bool {
    !contains_heredoc(command) && !has_stdin_redirect(command)
}

/// Maps to CC `rewriteWindowsNullRedirect(command)`.
pub fn rewrite_windows_null_redirect(command: &str) -> String {
    static NUL_REDIRECT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)(\d?&?>+\s*)nul").expect("valid nul redirect regex"));

    // Rust regex intentionally omits look-around, so check the source's trailing
    // boundary after every candidate and preserve non-matches byte-for-byte.
    let mut output = String::with_capacity(command.len());
    let mut last = 0;
    for matched in NUL_REDIRECT.find_iter(command) {
        let boundary = matched.end() == command.len()
            || command[matched.end()..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_whitespace() || "|&;)".contains(ch));
        if !boundary {
            continue;
        }
        output.push_str(&command[last..matched.start()]);
        let text = matched.as_str();
        output.push_str(&text[..text.len() - 3]);
        output.push_str("/dev/null");
        last = matched.end();
    }
    output.push_str(&command[last..]);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_and_stdin_redirect_match_official_shell_quoting() {
        assert_eq!(
            quote_shell_command("echo hi", true),
            "'echo hi' \\< /dev/null"
        );
        assert_eq!(
            quote_shell_command("cat <<'EOF'\nhello\nEOF", true),
            "'cat <<'\"'\"'EOF'\"'\"'\nhello\nEOF'"
        );
        assert!(has_stdin_redirect("cat < input.txt"));
        assert!(!has_stdin_redirect("cat <(printf hi)"));
        assert!(!should_add_stdin_redirect("cat <<EOF\nhi\nEOF"));
    }

    #[test]
    fn windows_nul_redirect_rewrite_matches_official_boundaries() {
        assert_eq!(rewrite_windows_null_redirect("ls 2>nul"), "ls 2>/dev/null");
        assert_eq!(
            rewrite_windows_null_redirect("x &>> NUL | cat"),
            "x &>> /dev/null | cat"
        );
        assert_eq!(rewrite_windows_null_redirect("echo >null"), "echo >null");
        assert_eq!(rewrite_windows_null_redirect("cat nul.txt"), "cat nul.txt");
    }
}
