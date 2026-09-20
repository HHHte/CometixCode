//! Maps to: CC `components/mcp/types.ts` and `MCPSettings.tsx` view state.

use crate::services::mcp::types::{
    ConfigScope, McpServerConnectionType, ScopedMcpServerConfig, Transport,
};

/// Maps to: CC `MCPToolDetailView`'s rendered JSON-schema property subset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpToolParameterInfo {
    pub name: String,
    pub type_name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// Maps to: CC `Tool` fields consumed by `MCPToolListView` and
/// `MCPToolDetailView`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpToolInfo {
    pub name: String,
    pub user_facing_name: Option<String>,
    pub description: Option<String>,
    pub is_read_only: bool,
    pub is_destructive: bool,
    pub is_open_world: bool,
    pub parameters: Vec<McpToolParameterInfo>,
}

/// Maps to: CC `MCPClientState` discriminated union in
/// `components/mcp/types.ts`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MCPClientState {
    Connected {
        tools: Vec<McpToolInfo>,
        resources_count: usize,
    },
    Pending,
    Failed {
        error: Option<String>,
    },
    Disabled,
    NeedsAuth {
        auth_url: Option<String>,
    },
}

impl MCPClientState {
    pub fn client_type(&self) -> McpServerConnectionType {
        match self {
            Self::Connected { .. } => McpServerConnectionType::Connected,
            Self::Pending => McpServerConnectionType::Pending,
            Self::Failed { .. } => McpServerConnectionType::Failed,
            Self::Disabled => McpServerConnectionType::Disabled,
            Self::NeedsAuth { .. } => McpServerConnectionType::NeedsAuth,
        }
    }
}

/// Maps to: CC `components/mcp/types.ts#MCPClientState` construction from
/// `AppState.mcp.clients` entries.
pub fn mcp_client_state_from_parts(
    client_type: McpServerConnectionType,
    tools: Vec<McpToolInfo>,
    resources_count: usize,
    error: Option<String>,
    auth_url: Option<String>,
) -> MCPClientState {
    match client_type {
        McpServerConnectionType::Connected => MCPClientState::Connected {
            tools,
            resources_count,
        },
        McpServerConnectionType::Pending => MCPClientState::Pending,
        McpServerConnectionType::Failed => MCPClientState::Failed { error },
        McpServerConnectionType::Disabled => MCPClientState::Disabled,
        McpServerConnectionType::NeedsAuth => MCPClientState::NeedsAuth { auth_url },
    }
}

/// Maps to: CC `ServerInfo` as built by `MCPSettings.prepareServers()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerInfo {
    pub name: String,
    /// Official `ServerInfo.client` state. `client_type`, `tools`, and
    /// `resources_count` below are retained as Rust rendering conveniences while
    /// preserving the official discriminated state as the UI source of truth.
    pub client: MCPClientState,
    pub client_type: McpServerConnectionType,
    pub scope: ConfigScope,
    pub transport: Transport,
    pub is_authenticated: Option<bool>,
    pub config: ScopedMcpServerConfig,
    pub reconnect_attempt: Option<u32>,
    pub max_reconnect_attempts: Option<u32>,
    /// Readonly counterpart of `filterToolsByServer(mcp.tools, server.name)`.
    pub tools: Vec<McpToolInfo>,
    /// Readonly counterpart of `filterMcpPromptsByServer(mcp.commands, ...)`.
    pub prompts_count: usize,
    /// Readonly counterpart of `mcp.resources[server.name]?.length`.
    pub resources_count: usize,
}

impl ServerInfo {
    /// Maps to: CC reads of `server.client.type` in the MCP settings panes.
    pub fn official_client_type(&self) -> McpServerConnectionType {
        self.client.client_type()
    }
}

/// Maps to: CC `AgentMcpServerInfo` for agent-only MCP entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMcpServerInfo {
    pub name: String,
    pub transport: Transport,
    pub url: Option<String>,
    pub command: Option<String>,
    pub source_agents: Vec<String>,
    pub needs_auth: bool,
    pub is_authenticated: bool,
}

/// Maps to: CC `MCPViewState` discriminated union.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MCPViewState {
    List {
        default_tab: Option<String>,
    },
    ServerMenu {
        server: ServerInfo,
    },
    ServerTools {
        server: ServerInfo,
    },
    ServerToolDetail {
        server: ServerInfo,
        tool_index: usize,
    },
    AgentServerMenu {
        agent_server: AgentMcpServerInfo,
    },
}

impl Default for MCPViewState {
    fn default() -> Self {
        Self::List { default_tab: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_client_state_discriminators_match_official_client_state() {
        let connected = mcp_client_state_from_parts(
            McpServerConnectionType::Connected,
            vec![McpToolInfo {
                name: "tool".to_string(),
                user_facing_name: None,
                description: None,
                is_read_only: true,
                is_destructive: false,
                is_open_world: false,
                parameters: Vec::new(),
            }],
            2,
            None,
            None,
        );
        assert_eq!(connected.client_type(), McpServerConnectionType::Connected);
        assert!(matches!(
            connected,
            MCPClientState::Connected {
                resources_count: 2,
                ..
            }
        ));

        let failed = mcp_client_state_from_parts(
            McpServerConnectionType::Failed,
            Vec::new(),
            0,
            Some("boom".to_string()),
            None,
        );
        assert_eq!(failed.client_type(), McpServerConnectionType::Failed);
        assert!(matches!(failed, MCPClientState::Failed { error: Some(_) }));

        let needs_auth = mcp_client_state_from_parts(
            McpServerConnectionType::NeedsAuth,
            Vec::new(),
            0,
            None,
            Some("https://auth.example".to_string()),
        );
        assert_eq!(needs_auth.client_type(), McpServerConnectionType::NeedsAuth);
        assert!(matches!(
            needs_auth,
            MCPClientState::NeedsAuth { auth_url: Some(_) }
        ));
    }
}
