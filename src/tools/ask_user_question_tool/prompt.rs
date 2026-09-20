//! Maps to CC `tools/AskUserQuestionTool/prompt.ts`.

pub const ASK_USER_QUESTION_TOOL_NAME: &str = "AskUserQuestion";

pub const ASK_USER_QUESTION_TOOL_CHIP_WIDTH: usize = 12;

pub const DESCRIPTION: &str = "Asks the user multiple choice questions to gather information, clarify ambiguity, understand preferences, make decisions or offer them choices.";

/// Maps to: CC `prompt.ts:11-20` `PREVIEW_FEATURE_PROMPT.markdown`.
pub const PREVIEW_FEATURE_PROMPT_MARKDOWN: &str = r#"
Preview feature:
Use the optional `preview` field on options when presenting concrete artifacts that users need to visually compare:
- ASCII mockups of UI layouts or components
- Code snippets showing different implementations
- Diagram variations
- Configuration examples

Preview content is rendered as markdown in a monospace box. Multi-line text with newlines is supported. When any option has a preview, the UI switches to a side-by-side layout with a vertical option list on the left and preview on the right. Do not use previews for simple preference questions where labels and descriptions suffice. Note: previews are only supported for single-select questions (not multiSelect).
"#;

/// Maps to: CC `prompt.ts:21-29` `PREVIEW_FEATURE_PROMPT.html`.
pub const PREVIEW_FEATURE_PROMPT_HTML: &str = r#"
Preview feature:
Use the optional `preview` field on options when presenting concrete artifacts that users need to visually compare:
- HTML mockups of UI layouts or components
- Formatted code snippets showing different implementations
- Visual comparisons or diagrams

Preview content must be a self-contained HTML fragment (no <html>/<body> wrapper, no <script> or <style> tags — use inline style attributes instead). Do not use previews for simple preference questions where labels and descriptions suffice. Note: previews are only supported for single-select questions (not multiSelect).
"#;

/// Maps to: CC `AskUserQuestionTool.tsx:207-215` `prompt()`. An SDK consumer
/// that never opted into a preview format gets no preview guidance, because it
/// may not render the field at all.
pub fn tool_prompt() -> String {
    match crate::bootstrap::state::get_question_preview_format() {
        None => ASK_USER_QUESTION_TOOL_PROMPT.to_string(),
        Some(crate::bootstrap::state::QuestionPreviewFormat::Markdown) => {
            format!("{ASK_USER_QUESTION_TOOL_PROMPT}{PREVIEW_FEATURE_PROMPT_MARKDOWN}")
        }
        Some(crate::bootstrap::state::QuestionPreviewFormat::Html) => {
            format!("{ASK_USER_QUESTION_TOOL_PROMPT}{PREVIEW_FEATURE_PROMPT_HTML}")
        }
    }
}

pub const ASK_USER_QUESTION_TOOL_PROMPT: &str = r#"Use this tool when you need to ask the user questions during execution. This allows you to:
1. Gather user preferences or requirements
2. Clarify ambiguous instructions
3. Get decisions on implementation choices as you work
4. Offer choices to the user about what direction to take.

Usage notes:
- Users will always be able to select "Other" to provide custom text input
- Use multiSelect: true to allow multiple answers to be selected for a question
- If you recommend a specific option, make that the first option in the list and add "(Recommended)" at the end of the label

Plan mode note: In plan mode, use this tool to clarify requirements or choose between approaches BEFORE finalizing your plan. Do NOT use this tool to ask "Is my plan ready?" or "Should I proceed?" - use ExitPlanMode for plan approval. IMPORTANT: Do not reference "the plan" in your questions (e.g., "Do you have feedback about the plan?", "Does the plan look good?") because the user cannot see the plan in the UI until you call ExitPlanMode. If you need plan approval, use ExitPlanMode instead.
"#;
