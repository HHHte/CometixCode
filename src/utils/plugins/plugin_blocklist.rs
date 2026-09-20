//! Maps to: CC `utils/plugins/pluginBlocklist.ts`.
use super::{installed_plugins_manager, marketplace_manager, plugin_flagging};
use serde_json::Value;

/// Maps to: CC `pluginBlocklist.ts#detectDelistedPlugins`.
pub fn detect_delisted_plugins(
    installed_plugins: &Value,
    marketplace: &Value,
    marketplace_name: &str,
) -> Vec<String> {
    let names: std::collections::HashSet<&str> = marketplace["plugins"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["name"].as_str())
        .collect();
    let suffix = format!("@{marketplace_name}");
    installed_plugins["plugins"]
        .as_object()
        .map(|plugins| {
            crate::utils::process_env::ecmascript_object_entries(plugins)
                .into_iter()
                .filter_map(|(id, _)| {
                    id.strip_suffix(&suffix)
                        .filter(|name| !names.contains(name))
                        .map(|_| id.to_owned())
                })
                .collect()
        })
        .unwrap_or_default()
}
/// Maps to: CC `pluginBlocklist.ts#detectAndUninstallDelistedPlugins`.
pub async fn detect_and_uninstall_delisted_plugins() -> Vec<String> {
    plugin_flagging::load_flagged_plugins().await;
    let installed = installed_plugins_manager::load_installed_plugins_v2();
    let already_flagged = plugin_flagging::get_flagged_plugins();
    let known = marketplace_manager::load_known_marketplaces_config_safe().await;
    let mut newly_flagged = Vec::new();
    for (name, _) in crate::utils::process_env::ecmascript_object_entries(&known) {
        match marketplace_manager::get_marketplace(name).await {
            Ok(marketplace) => {
                if marketplace["forceRemoveDeletedPlugins"] != true {
                    continue;
                }
                let delisted =
                    detect_delisted_plugins(&installed.lock().unwrap(), &marketplace, name);
                for id in delisted {
                    if already_flagged.contains_key(&id) {
                        continue;
                    }
                    let installations = installed.lock().unwrap()["plugins"][&id]
                        .as_array()
                        .cloned()
                        .unwrap_or_default();
                    if !installations
                        .iter()
                        .any(|i| matches!(i["scope"].as_str(), Some("user" | "project" | "local")))
                    {
                        continue;
                    }
                    for installation in installations {
                        let scope = match installation["scope"].as_str() {
                            Some("user") => super::schemas::PluginScope::User,
                            Some("project") => super::schemas::PluginScope::Project,
                            Some("local") => super::schemas::PluginScope::Local,
                            _ => continue,
                        };
                        if let Err(error) =
                            crate::services::plugins::plugin_operations::uninstall_plugin_op(
                                &id, scope, false,
                            )
                            .await
                        {
                            crate::utils::debug::log_for_debugging(&format!(
                                "Failed to auto-uninstall delisted plugin {id} from {}: {error}",
                                installation["scope"].as_str().unwrap()
                            ));
                        }
                    }
                    plugin_flagging::add_flagged_plugin(&id).await;
                    newly_flagged.push(id);
                }
            }
            Err(error) => crate::utils::debug::log_for_debugging(&format!(
                "Failed to check for delisted plugins in \"{name}\": {error}"
            )),
        }
    }
    newly_flagged
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delisted_detection_preserves_suffix_case_and_installed_order() {
        let installed = serde_json::json!({"plugins":{"gone@m":[],"keep@m":[],"gone@M":[],"x@other":[],"nested@id@m":[]}});
        let marketplace = serde_json::json!({"plugins":[{"name":"keep"}]});
        assert_eq!(
            detect_delisted_plugins(&installed, &marketplace, "m"),
            ["gone@m", "nested@id@m"]
        );
    }
}
