//! Built-in plugin registry.
//!
//! Maps to: CC `plugins/builtinPlugins.ts`.

use crate::types::plugin::LoadedPlugin;
use crate::utils::plugins::schemas::PluginManifest;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Maps to CC `plugins/builtinPlugins.ts#BUILTIN_MARKETPLACE_NAME`.
pub const BUILTIN_MARKETPLACE_NAME: &str = "builtin";

/// Definition for a built-in plugin that ships with the CLI.
///
/// Maps to: CC `types/plugin.ts#BuiltinPluginDefinition`.
#[derive(Clone, Debug)]
pub struct BuiltinPluginDefinition {
    /// Maps to CC `BuiltinPluginDefinition.name`.
    pub name: String,
    /// Maps to CC `BuiltinPluginDefinition.description`.
    pub description: String,
    /// Maps to CC `BuiltinPluginDefinition.version`.
    pub version: Option<String>,
    /// Maps to CC `BuiltinPluginDefinition.hooks`.
    pub hooks: Option<Value>,
    /// Maps to CC `BuiltinPluginDefinition.mcpServers`.
    pub mcp_servers: Option<Value>,
    /// Maps to CC `BuiltinPluginDefinition.isAvailable`.
    pub is_available: Option<fn() -> bool>,
    /// Maps to CC `BuiltinPluginDefinition.defaultEnabled`.
    pub default_enabled: Option<bool>,
}

impl BuiltinPluginDefinition {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            version: None,
            hooks: None,
            mcp_servers: None,
            is_available: None,
            default_enabled: None,
        }
    }
}

/// Return shape for enabled/disabled built-in plugins.
///
/// Maps to CC `plugins/builtinPlugins.ts#getBuiltinPlugins`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuiltinPluginsResult {
    pub enabled: Vec<LoadedPlugin>,
    pub disabled: Vec<LoadedPlugin>,
}

static BUILTIN_PLUGINS: OnceLock<Mutex<BTreeMap<String, BuiltinPluginDefinition>>> =
    OnceLock::new();

fn registry() -> &'static Mutex<BTreeMap<String, BuiltinPluginDefinition>> {
    BUILTIN_PLUGINS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Register a built-in plugin.
///
/// Maps to: CC `plugins/builtinPlugins.ts#registerBuiltinPlugin`.
pub fn register_builtin_plugin(definition: BuiltinPluginDefinition) {
    registry()
        .lock()
        .expect("builtin plugin registry poisoned")
        .insert(definition.name.clone(), definition);
}

/// Check whether a plugin id uses the built-in marketplace sentinel.
///
/// Maps to: CC `plugins/builtinPlugins.ts#isBuiltinPluginId`.
pub fn is_builtin_plugin_id(plugin_id: &str) -> bool {
    plugin_id.ends_with(&format!("@{BUILTIN_MARKETPLACE_NAME}"))
}

/// Get a registered built-in plugin definition by name.
///
/// Maps to: CC `plugins/builtinPlugins.ts#getBuiltinPluginDefinition`.
pub fn get_builtin_plugin_definition(name: &str) -> Option<BuiltinPluginDefinition> {
    registry()
        .lock()
        .expect("builtin plugin registry poisoned")
        .get(name)
        .cloned()
}

/// Get all registered built-in plugins split by enabled state.
///
/// Maps to: CC `plugins/builtinPlugins.ts#getBuiltinPlugins`.
pub fn get_builtin_plugins() -> BuiltinPluginsResult {
    let settings = crate::utils::settings::get_initial_settings();
    let definitions = registry()
        .lock()
        .expect("builtin plugin registry poisoned")
        .clone();
    get_builtin_plugins_from_settings_and_definitions(&settings, definitions.values())
}

fn get_builtin_plugins_from_settings_and_definitions<'a>(
    settings: &crate::utils::settings::types::SettingsJson,
    definitions: impl IntoIterator<Item = &'a BuiltinPluginDefinition>,
) -> BuiltinPluginsResult {
    let mut result = BuiltinPluginsResult::default();
    let enabled_plugins = settings.enabled_plugins.as_ref().and_then(Value::as_object);

    for definition in definitions {
        if definition
            .is_available
            .is_some_and(|is_available| !is_available())
        {
            continue;
        }

        let plugin_id = format!("{}@{BUILTIN_MARKETPLACE_NAME}", definition.name);
        let user_setting = enabled_plugins
            .and_then(|plugins| plugins.get(&plugin_id))
            .and_then(Value::as_bool);
        // Maps to CC enabled-state precedence:
        // `settings.enabledPlugins[pluginId] ?? definition.defaultEnabled ?? true`.
        let enabled = user_setting.unwrap_or(definition.default_enabled.unwrap_or(true));

        let plugin = LoadedPlugin {
            name: definition.name.clone(),
            manifest: PluginManifest {
                name: definition.name.clone(),
                description: Some(definition.description.clone()),
                version: definition.version.clone(),
                agents: None,
                user_config: None,
                hooks: definition.hooks.clone(),
                mcp_servers: definition.mcp_servers.clone(),
                lsp_servers: None,
                channels: None,
                ..Default::default()
            },
            path: std::path::PathBuf::from(BUILTIN_MARKETPLACE_NAME),
            source: plugin_id.clone(),
            agents_path: None,
            agents_paths: Vec::new(),
            enabled,
            is_builtin: true,
            hooks_config: definition.hooks.clone(),
            mcp_servers: definition.mcp_servers.clone().into(),
            lsp_servers: Default::default(),
            ..Default::default()
        };

        if enabled {
            result.enabled.push(plugin);
        } else {
            result.disabled.push(plugin);
        }
    }

    result
}

/// Clear the built-in plugin registry.
///
/// Maps to: CC `plugins/builtinPlugins.ts#clearBuiltinPlugins`.
pub fn clear_builtin_plugins() {
    registry()
        .lock()
        .expect("builtin plugin registry poisoned")
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unavailable() -> bool {
        false
    }

    fn settings_with_enabled_plugins(value: Value) -> crate::utils::settings::types::SettingsJson {
        crate::utils::settings::types::SettingsJson {
            enabled_plugins: Some(value),
            ..Default::default()
        }
    }

    #[test]
    fn builtin_plugin_id_uses_official_builtin_marketplace_suffix() {
        assert!(is_builtin_plugin_id("reviewer@builtin"));
        assert!(!is_builtin_plugin_id("reviewer@marketplace"));
    }

    #[test]
    fn get_builtin_plugins_honors_settings_default_and_availability() {
        let mut default_enabled = BuiltinPluginDefinition::new("default-on", "enabled by default");
        default_enabled.version = Some("1.2.3".to_string());
        default_enabled.hooks = Some(serde_json::json!({"PreToolUse": []}));
        default_enabled.mcp_servers = Some(serde_json::json!({"server": {"type": "stdio"}}));

        let mut default_disabled =
            BuiltinPluginDefinition::new("default-off", "disabled by default");
        default_disabled.default_enabled = Some(false);

        let mut forced_on = BuiltinPluginDefinition::new("forced-on", "enabled by settings");
        forced_on.default_enabled = Some(false);

        let mut forced_off = BuiltinPluginDefinition::new("forced-off", "disabled by settings");
        forced_off.default_enabled = Some(true);

        let mut hidden = BuiltinPluginDefinition::new("hidden", "unavailable");
        hidden.is_available = Some(unavailable);

        let settings = settings_with_enabled_plugins(serde_json::json!({
            "forced-on@builtin": true,
            "forced-off@builtin": false
        }));
        let result = get_builtin_plugins_from_settings_and_definitions(
            &settings,
            [
                &default_enabled,
                &default_disabled,
                &forced_on,
                &forced_off,
                &hidden,
            ],
        );

        let enabled = result
            .enabled
            .iter()
            .map(|plugin| plugin.source.as_str())
            .collect::<Vec<_>>();
        let disabled = result
            .disabled
            .iter()
            .map(|plugin| plugin.source.as_str())
            .collect::<Vec<_>>();

        assert_eq!(enabled, vec!["default-on@builtin", "forced-on@builtin"]);
        assert_eq!(disabled, vec!["default-off@builtin", "forced-off@builtin"]);
        let plugin = &result.enabled[0];
        assert!(plugin.is_builtin);
        assert_eq!(plugin.path, std::path::PathBuf::from("builtin"));
        assert_eq!(
            plugin.manifest.description.as_deref(),
            Some("enabled by default")
        );
        assert_eq!(plugin.manifest.version.as_deref(), Some("1.2.3"));
        assert!(plugin.hooks_config.is_some());
        assert!(plugin.mcp_servers.is_some());
    }

    #[test]
    fn register_and_clear_builtin_plugin_registry_matches_official_helpers() {
        clear_builtin_plugins();
        register_builtin_plugin(BuiltinPluginDefinition::new("alpha", "alpha plugin"));
        assert_eq!(
            get_builtin_plugin_definition("alpha").map(|definition| definition.description),
            Some("alpha plugin".to_string())
        );
        clear_builtin_plugins();
        assert!(get_builtin_plugin_definition("alpha").is_none());
    }
}
