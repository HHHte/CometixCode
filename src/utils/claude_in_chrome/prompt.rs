//! Claude in Chrome prompt fragments.
//!
//! Maps to CC `utils/claudeInChrome/prompt.ts`.

/// Maps to CC `utils/claudeInChrome/prompt.ts:53-62`
/// `CHROME_TOOL_SEARCH_INSTRUCTIONS`.
pub const CHROME_TOOL_SEARCH_INSTRUCTIONS: &str = r#"**IMPORTANT: Before using any chrome browser tools, you MUST first load them using ToolSearch.**

Chrome browser tools are MCP tools that require loading before use. Before calling any mcp__claude-in-chrome__* tool:
1. Use ToolSearch with `select:mcp__claude-in-chrome__<tool_name>` to load the specific tool
2. Then call the tool

For example, to get tab context:
1. First: ToolSearch with query "select:mcp__claude-in-chrome__tabs_context_mcp"
2. Then: Call mcp__claude-in-chrome__tabs_context_mcp"#;
