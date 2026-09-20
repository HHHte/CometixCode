//! Maps to: CC `tools/BashTool/commentLabel.ts`.

/// Maps to CC `extractBashCommentLabel(command)`.
pub fn extract_bash_comment_label(command: &str) -> Option<String> {
    let first_line = command
        .split_once('\n')
        .map(|(first, _)| first)
        .unwrap_or(command)
        .trim();
    if !first_line.starts_with('#') || first_line.starts_with("#!") {
        return None;
    }
    let label = first_line.trim_start_matches('#').trim_start().to_string();
    (!label.is_empty()).then_some(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_first_line_comment_label_but_not_shebang() {
        assert_eq!(
            extract_bash_comment_label("# Check status\ngit status").as_deref(),
            Some("Check status")
        );
        assert_eq!(
            extract_bash_comment_label("###   Deploy service\necho ok").as_deref(),
            Some("Deploy service")
        );
        assert!(extract_bash_comment_label("#!/usr/bin/env bash\necho ok").is_none());
        assert!(extract_bash_comment_label("echo '# not first line'").is_none());
        assert!(extract_bash_comment_label("#   ").is_none());
    }
}
