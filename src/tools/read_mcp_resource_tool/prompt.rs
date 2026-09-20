//! Maps to: CC `tools/ReadMcpResourceTool/prompt.ts`.

/// Maps to: CC `tools/ReadMcpResourceTool/ReadMcpResourceTool.ts:60` `name`,
/// which official declares inline rather than in `prompt.ts`.
pub const READ_MCP_RESOURCE_TOOL_NAME: &str = "ReadMcpResourceTool";

pub const READ_MCP_RESOURCE_DESCRIPTION: &str = r#"
Reads a specific resource from an MCP server.
- server: The name of the MCP server to read from
- uri: The URI of the resource to read

Usage examples:
- Read a resource from a server: `readMcpResource({ server: "myserver", uri: "my-resource-uri" })`
"#;

pub const READ_MCP_RESOURCE_PROMPT: &str = r#"
Reads a specific resource from an MCP server, identified by server name and resource URI.

Parameters:
- server (required): The name of the MCP server from which to read the resource
- uri (required): The URI of the resource to read
"#;
