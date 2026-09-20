//! Maps to: CC commands/plugin/unifiedTypes.ts#UnifiedInstalledItem.
//! The reconstructed type declaration is incomplete; fields and discriminants
//! below are evidenced by ManagePlugins.tsx:696-948 and UnifiedInstalledCell.tsx.
use crate::components::mcp::types::ServerInfo;
use crate::types::plugin::{LoadedPlugin, PluginError};
#[derive(Clone, Debug, PartialEq)]
pub struct UnifiedInstalledItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub marketplace: Option<String>,
    pub scope: String,
    pub kind: UnifiedInstalledKind,
}
#[derive(Clone, Debug, PartialEq)]
pub enum UnifiedInstalledKind {
    Plugin {
        plugin: LoadedPlugin,
        is_enabled: bool,
        errors: Vec<PluginError>,
        pending_enable: Option<bool>,
        pending_update: bool,
        pending_toggle: Option<String>,
    },
    FailedPlugin {
        errors: Vec<PluginError>,
    },
    FlaggedPlugin {
        reason: String,
        text: String,
        flagged_at: String,
    },
    Mcp {
        client: ServerInfo,
        indented: bool,
    },
}
