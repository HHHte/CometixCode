//! Compaction prompts and summary formatting.
//! Maps to CC `services/compact/prompt.ts`.

const COMPACT_PROMPT_TEMPLATE: &str = include_str!("base_compact_prompt.txt");

/// Maps to CC `getCompactPrompt(...)`.
pub fn get_compact_prompt(custom_instructions: Option<&str>) -> String {
    let custom = custom_instructions
        .filter(|instructions| !instructions.trim().is_empty())
        .map(|instructions| format!("\n\nAdditional Instructions:\n{instructions}"))
        .unwrap_or_default();
    COMPACT_PROMPT_TEMPLATE.replace("\n{{CUSTOM_INSTRUCTIONS}}", &custom)
}

/// Maps to CC prompt.ts#getPartialCompactPrompt. Template text is copied from CC 2.1.88.
pub fn get_partial_compact_prompt(
    custom_instructions: Option<&str>,
    direction: crate::types::message::PartialCompactDirection,
) -> String {
    let template = match direction {
        crate::types::message::PartialCompactDirection::From => {
            include_str!("partial_compact_from_prompt.txt")
        }
        crate::types::message::PartialCompactDirection::UpTo => {
            include_str!("partial_compact_up_to_prompt.txt")
        }
    };
    let custom = custom_instructions
        .filter(|text| !text.trim().is_empty())
        .map(|text| format!("\n\nAdditional Instructions:\n{text}"))
        .unwrap_or_default();
    template.replace("{{CUSTOM_INSTRUCTIONS}}", &custom)
}

/// Maps to CC `formatCompactSummary(...)`.
pub fn format_compact_summary(summary: &str) -> String {
    let without_analysis = strip_first_tagged_section(summary, "analysis");
    let formatted = replace_first_summary_section(&without_analysis);
    collapse_blank_lines(&formatted).trim().to_string()
}

/// Maps to CC `getCompactUserSummaryMessage(...)`.
pub fn get_compact_user_summary_message(
    summary: &str,
    suppress_follow_up_questions: bool,
    transcript_path: Option<&str>,
    recent_messages_preserved: bool,
) -> String {
    let mut base = format!(
        "This session is being continued from a previous conversation that ran out of context. The summary below covers the earlier portion of the conversation.\n\n{}",
        format_compact_summary(summary)
    );
    if let Some(path) = transcript_path.filter(|path| !path.is_empty()) {
        base.push_str("\n\nIf you need specific details from before compaction (like exact code snippets, error messages, or content you generated), read the full transcript at: ");
        base.push_str(path);
    }
    if recent_messages_preserved {
        base.push_str("\n\nRecent messages are preserved verbatim.");
    }
    if suppress_follow_up_questions {
        base.push_str(
            "\nContinue the conversation from where it left off without asking the user any further questions. Resume directly — do not acknowledge the summary, do not recap what was happening, do not preface with \"I'll continue\" or similar. Pick up the last task as if the break never happened.",
        );
    }
    base
}

fn strip_first_tagged_section(input: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let Some(start) = input.find(&open) else {
        return input.to_string();
    };
    let Some(relative_end) = input[start + open.len()..].find(&close) else {
        return input.to_string();
    };
    let end = start + open.len() + relative_end + close.len();
    format!("{}{}", &input[..start], &input[end..])
}

fn replace_first_summary_section(input: &str) -> String {
    let Some(start) = input.find("<summary>") else {
        return input.to_string();
    };
    let body_start = start + "<summary>".len();
    let Some(relative_end) = input[body_start..].find("</summary>") else {
        return input.to_string();
    };
    let body_end = body_start + relative_end;
    let end = body_end + "</summary>".len();
    format!(
        "{}Summary:\n{}{}",
        &input[..start],
        input[body_start..body_end].trim(),
        &input[end..]
    )
}

fn collapse_blank_lines(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut consecutive_newlines = 0usize;
    for ch in input.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                out.push(ch);
            }
        } else {
            consecutive_newlines = 0;
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_prompt_matches_official_no_tools_and_custom_instruction_shape() {
        let prompt = get_compact_prompt(Some("Focus on Rust changes"));
        assert!(prompt.starts_with("CRITICAL: Respond with TEXT ONLY."));
        assert!(prompt.contains("Additional Instructions:\nFocus on Rust changes"));
        assert!(prompt.ends_with("Tool calls will be rejected and you will fail the task."));
    }

    #[test]
    fn compact_summary_strips_analysis_and_formats_summary_like_official() {
        assert_eq!(
            format_compact_summary(
                "<analysis>draft</analysis>\n\n<summary>\nUseful context\n</summary>"
            ),
            "Summary:\nUseful context"
        );
    }
    #[test]
    fn partial_prompt_matches_official_direction_and_instruction_order() {
        use crate::types::message::PartialCompactDirection;
        // CC prompt.ts:274-294, external text is copied verbatim to the two templates.
        let from =
            get_partial_compact_prompt(Some("  keep decisions  "), PartialCompactDirection::From);
        assert!(from.starts_with("CRITICAL: Respond with TEXT ONLY. Do NOT call any tools."));
        assert!(from.contains("RECENT portion of the conversation"));
        assert!(from.contains("\n\nAdditional Instructions:\n  keep decisions  \n\nREMINDER:"));
        let up = get_partial_compact_prompt(Some(" "), PartialCompactDirection::UpTo);
        assert!(
            up.contains("newer messages that build on this context will follow after your summary")
        );
        assert!(!up.contains("Additional Instructions:"));
        assert!(up.ends_with("Tool calls will be rejected and you will fail the task."));
    }
}
