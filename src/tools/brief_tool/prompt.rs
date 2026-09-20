//! Brief tool prompt constants.
//! Maps to CC `tools/BriefTool/prompt.ts`.

pub const BRIEF_TOOL_NAME: &str = "SendUserMessage";
pub const LEGACY_BRIEF_TOOL_NAME: &str = "Brief";
pub const DESCRIPTION: &str = "Send a message to the user";

/// Maps to: CC `tools/BriefTool/prompt.ts:12-22` `BRIEF_PROACTIVE_SECTION`.
/// Injected as the `brief` system prompt section (`constants/prompts.ts:553`).
pub fn brief_proactive_section() -> String {
    format!(
        "## Talking to the user\n\n{BRIEF_TOOL_NAME} is where your replies go. Text outside it is visible if the user expands the detail view, but most won't — assume unread. Anything you want them to actually see goes through {BRIEF_TOOL_NAME}. The failure mode: the real answer lives in plain text while {BRIEF_TOOL_NAME} just says \"done!\" — they see \"done!\" and miss everything.\n\nSo: every time the user says something, the reply they actually read comes through {BRIEF_TOOL_NAME}. Even for \"hi\". Even for \"thanks\".\n\nIf you can answer right away, send the answer. If you need to go look — run a command, read files, check something — ack first in one line (\"On it — checking the test output\"), then work, then send the result. Without the ack they're staring at a spinner.\n\nFor longer work: ack → work → result. Between those, send a checkpoint when something useful happened — a decision you made, a surprise you hit, a phase boundary. Skip the filler (\"running tests...\") — a checkpoint earns its place by carrying information.\n\nKeep messages tight — the decision, the file:line, the PR number. Second person always (\"your config\"), never third."
    )
}

pub const BRIEF_TOOL_PROMPT: &str = "Send a message the user will read. Text outside this tool is visible in the detail view, but most won't open it — the answer lives here.\n\n`message` supports markdown. `attachments` takes file paths (absolute or cwd-relative) for images, diffs, logs.\n\n`status` labels intent: 'normal' when replying to what they just asked; 'proactive' when you're initiating — a scheduled task finished, a blocker surfaced during background work, you need input on something they haven't asked about. Set it honestly; downstream routing uses it.";
