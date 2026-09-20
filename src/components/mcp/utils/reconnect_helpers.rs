//! Maps to: CC `components/mcp/utils/reconnectHelpers.tsx`.

use crate::services::mcp::types::McpServerConnectionType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconnectResult {
    pub message: String,
    pub success: bool,
}

/// Maps to: CC `handleReconnectResult`.
pub fn handle_reconnect_result(
    client_type: McpServerConnectionType,
    server_name: &str,
) -> ReconnectResult {
    match client_type {
        McpServerConnectionType::Connected => ReconnectResult {
            message: format!("Reconnected to {server_name}."),
            success: true,
        },
        McpServerConnectionType::NeedsAuth => ReconnectResult {
            message: format!(
                "{server_name} requires authentication. Use the 'Authenticate' option."
            ),
            success: false,
        },
        McpServerConnectionType::Failed => ReconnectResult {
            message: format!("Failed to reconnect to {server_name}."),
            success: false,
        },
        McpServerConnectionType::Pending | McpServerConnectionType::Disabled => ReconnectResult {
            message: format!("Unknown result when reconnecting to {server_name}."),
            success: false,
        },
    }
}

/// Maps to: CC `handleReconnectError`.
pub fn handle_reconnect_error(error: impl std::fmt::Display, server_name: &str) -> String {
    format!("Error reconnecting to {server_name}: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnect_helpers_match_official_menu_copy() {
        assert_eq!(
            handle_reconnect_result(McpServerConnectionType::Connected, "docs"),
            ReconnectResult {
                message: "Reconnected to docs.".to_string(),
                success: true,
            }
        );
        assert_eq!(
            handle_reconnect_result(McpServerConnectionType::NeedsAuth, "docs"),
            ReconnectResult {
                message: "docs requires authentication. Use the 'Authenticate' option.".to_string(),
                success: false,
            }
        );
        assert_eq!(
            handle_reconnect_result(McpServerConnectionType::Failed, "docs"),
            ReconnectResult {
                message: "Failed to reconnect to docs.".to_string(),
                success: false,
            }
        );
        assert_eq!(
            handle_reconnect_result(McpServerConnectionType::Pending, "docs"),
            ReconnectResult {
                message: "Unknown result when reconnecting to docs.".to_string(),
                success: false,
            }
        );
        assert_eq!(
            handle_reconnect_error("boom", "docs"),
            "Error reconnecting to docs: boom"
        );
    }
}
