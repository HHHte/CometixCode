//! Managed plugin helpers.
//!
//! Maps to: CC `utils/plugins/managedPlugins.ts`.

use crate::utils::settings::types::SettingsJson;
use serde_json::Value;
use std::collections::HashSet;

/// Plugin names locked by enterprise managed settings.
///
/// Maps to: CC `utils/plugins/managedPlugins.ts#getManagedPluginNames`.
/// Only `enabledPlugins` entries whose value is boolean and whose key is a
/// `plugin@marketplace` id are protected. Both `true` and `false` represent
/// admin intent and block session `--plugin-dir` overrides.
pub fn get_managed_plugin_names() -> Option<HashSet<String>> {
    let settings = crate::utils::settings::load_settings_from_disk();
    let policy_settings = settings.policy_settings.as_ref()?;
    managed_plugin_names_from_settings(policy_settings)
}

fn managed_plugin_names_from_settings(settings: &SettingsJson) -> Option<HashSet<String>> {
    let enabled_plugins = settings.enabled_plugins.as_ref()?.as_object()?;
    let mut names = HashSet::new();
    for (plugin_id, value) in enabled_plugins {
        if !matches!(value, Value::Bool(_)) || !plugin_id.contains('@') {
            continue;
        }
        let name = plugin_id.split('@').next().unwrap_or_default();
        if !name.is_empty() {
            names.insert(name.to_string());
        }
    }
    (!names.is_empty()).then_some(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_plugin_names_match_policy_boolean_plugin_ids_only() {
        let settings = SettingsJson {
            enabled_plugins: Some(serde_json::json!({
                "alpha@market": true,
                "beta@market": false,
                "legacy-owner-repo": ["repo"],
                "gamma@market": {"enabled": true},
                "": true,
                "@market": true,
            })),
            ..SettingsJson::default()
        };

        let names = managed_plugin_names_from_settings(&settings).expect("names");
        assert!(names.contains("alpha"));
        assert!(names.contains("beta"));
        assert!(!names.contains("legacy-owner-repo"));
        assert!(!names.contains("gamma"));
        assert!(!names.contains(""));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn managed_plugin_names_returns_none_without_policy_entries() {
        let settings = SettingsJson {
            enabled_plugins: Some(serde_json::json!({
                "legacy-owner-repo": ["repo"],
                "gamma@market": {"enabled": true}
            })),
            ..SettingsJson::default()
        };

        assert!(managed_plugin_names_from_settings(&settings).is_none());
        assert!(managed_plugin_names_from_settings(&SettingsJson::default()).is_none());
    }
}
