//! Maps to: CC `components/MCPServerDialogCopy.tsx`.

use iocraft::prelude::*;

pub const MCP_SERVER_DOCS_URL: &str = "https://code.claude.com/docs/en/mcp";
pub const MCP_SERVER_DIALOG_COPY: &str = "MCP servers may execute code or access system resources. All tool calls require approval. Learn more in the";

#[component]
pub fn MCPServerDialogCopy() -> impl Into<AnyElement<'static>> {
    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(content: MCP_SERVER_DIALOG_COPY.to_string(), wrap: TextWrap::Wrap)
            Link(url: MCP_SERVER_DOCS_URL.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_dialog_copy_matches_official_text_and_link() {
        let text = element! { MCPServerDialogCopy() }
            .render(Some(100))
            .to_string();
        assert!(
            text.contains("MCP servers may execute code"),
            "canvas=\n{text}"
        );
        assert!(text.contains(MCP_SERVER_DOCS_URL), "canvas=\n{text}");
    }
}
