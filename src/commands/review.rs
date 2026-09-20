//! Maps to: CC `commands/review.ts` (local review descriptor and prompt).
use crate::types::message::UserContent;
/// Maps to CC `LOCAL_REVIEW_PROMPT`.
pub fn local_review_prompt(args: &str) -> String {
    format!(
        r#"
      You are an expert code reviewer. Follow these steps:

      1. If no PR number is provided in the args, run `gh pr list` to show open PRs
      2. If a PR number is provided, run `gh pr view <number>` to get PR details
      3. Run `gh pr diff <number>` to get the diff
      4. Analyze the changes and provide a thorough code review that includes:
         - Overview of what the PR does
         - Analysis of code quality and style
         - Specific suggestions for improvements
         - Any potential issues or risks

      Keep your review concise but thorough. Focus on:
      - Code correctness
      - Following project conventions
      - Performance implications
      - Test coverage
      - Security considerations

      Format your review with clear sections and bullet points.

      PR number: {args}
    "#
    )
}
/// Maps to CC `review.getPromptForCommand`.
pub fn get_prompt_for_command(
    _command: &super::Command,
    args: &str,
    _context: &crate::tool::ToolUseContext,
) -> anyhow::Result<Vec<UserContent>> {
    Ok(vec![UserContent::Text(local_review_prompt(args))])
}
pub fn command() -> super::Command {
    super::Command::prompt("review", "Review a pull request")
        .prompt_metadata("reviewing pull request", 0)
        .prompt_executable(get_prompt_for_command)
}
#[cfg(test)]
mod tests {
    #[test]
    fn local_review_matches_official_prompt_without_remote_path() {
        // CC commands/review.ts:9-31: local gh prompt; ultrareview is separate.
        let prompt = super::local_review_prompt("42");
        assert!(prompt.starts_with("\n      You are an expert code reviewer."));
        assert!(prompt.contains("`gh pr diff <number>`"));
        assert!(prompt.ends_with("PR number: 42\n    "));
        assert!(!prompt.contains("ultrareview"));
        assert!(super::command().get_prompt_for_command.is_some());
    }
}
