//! Maps to: CC `utils/plugins/pluginAutoupdate.ts`.
//! User-initiated marketplace update slice. Background notification lifecycle is
//! outside this slice; both paths share the canonical update functions below.
use super::installed_plugins_manager::{
    is_installation_relevant_to_current_project, load_installed_plugins_from_disk,
};
use super::plugin_identifier::parse_plugin_identifier;
use super::schemas::PluginScope;
use crate::utils::debug::{DebugLogLevel, log_for_debugging, log_for_debugging_with_level};
use serde_json::Value;
use std::collections::HashSet;

/// Maps to: CC `utils/plugins/pluginAutoupdate.ts:108-138#updatePlugin`.
async fn update_plugin(plugin_id: &str, installations: &[Value]) -> Option<String> {
    let mut was_updated = false;
    for installation in installations {
        let result = async {
            let scope: PluginScope = serde_json::from_value(installation["scope"].clone())?;
            #[cfg(test)]
            if let Ok(fixture) = AUTOUPDATE_TEST_IMPORTS.try_with(Clone::clone) {
                fixture
                    .calls
                    .lock()
                    .unwrap()
                    .push((plugin_id.to_owned(), scope));
                tokio::task::yield_now().await;
                if plugin_id.starts_with("error@") {
                    anyhow::bail!("fixture");
                }
                return Ok(
                    crate::services::plugins::plugin_operations::PluginUpdateResult {
                        success: true,
                        message: String::new(),
                        plugin_id: Some(plugin_id.into()),
                        old_version: Some("1".into()),
                        new_version: Some("2".into()),
                        already_up_to_date: Some(plugin_id.starts_with("already@")),
                        scope: Some(scope),
                    },
                );
            }
            crate::services::plugins::plugin_operations::update_plugin_op(plugin_id, scope).await
        }
        .await;
        match result {
            Ok(result) if result.success && result.already_up_to_date != Some(true) => {
                was_updated = true;
                log_for_debugging(&format!(
                    "Plugin autoupdate: updated {plugin_id} from {} to {}",
                    result.old_version.as_deref().unwrap_or("undefined"),
                    result.new_version.as_deref().unwrap_or("undefined")
                ));
            }
            Ok(result) if result.already_up_to_date != Some(true) => log_for_debugging_with_level(
                &format!(
                    "Plugin autoupdate: failed to update {plugin_id}: {}",
                    result.message
                ),
                DebugLogLevel::Warn,
            ),
            Ok(_) => {}
            Err(error) => log_for_debugging_with_level(
                &format!("Plugin autoupdate: error updating {plugin_id}: {error}"),
                DebugLogLevel::Warn,
            ),
        }
    }
    was_updated.then(|| plugin_id.to_owned())
}

/// Maps to: CC `utils/plugins/pluginAutoupdate.ts:161-200#updatePluginsForMarketplaces`.
/// A1/A3: poll all plugin futures concurrently, collect settled values in source
/// Object.keys order. Native errors are caught by updatePlugin per installation.
pub async fn update_plugins_for_marketplaces(marketplace_names: &HashSet<String>) -> Vec<String> {
    #[cfg(not(test))]
    let installed_plugins = load_installed_plugins_from_disk();
    #[cfg(test)]
    let installed_plugins = AUTOUPDATE_TEST_IMPORTS
        .try_with(|fixture| fixture.installed.clone())
        .unwrap_or_else(|_| load_installed_plugins_from_disk());
    let Some(plugins) = installed_plugins["plugins"].as_object() else {
        return Vec::new();
    };
    futures::future::join_all(
        crate::utils::process_env::ecmascript_object_entries(plugins)
            .into_iter()
            .map(|(plugin_id, all_installations)| async move {
                let parsed = parse_plugin_identifier(plugin_id);
                let marketplace = parsed.marketplace.filter(|name| !name.is_empty())?;
                if !marketplace_names.contains(&marketplace.to_lowercase()) {
                    return None;
                }
                let all_installations = all_installations.as_array()?;
                if all_installations.is_empty() {
                    return None;
                }
                let relevant: Vec<_> = all_installations
                    .iter()
                    .filter(|value| is_installation_relevant_to_current_project(value))
                    .cloned()
                    .collect();
                if relevant.is_empty() {
                    return None;
                }
                update_plugin(plugin_id, &relevant).await
            }),
    )
    .await
    .into_iter()
    .flatten()
    .collect()
}

// Imported I/O boundary only; tests retain the source filtering, concurrent
// polling, scope iteration, update result handling and final source-order fold.
#[cfg(test)]
#[derive(Clone)]
struct AutoUpdateTestImports {
    installed: Value,
    calls: std::sync::Arc<std::sync::Mutex<Vec<(String, PluginScope)>>>,
}
#[cfg(test)]
tokio::task_local! {static AUTOUPDATE_TEST_IMPORTS:AutoUpdateTestImports;}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn marketplace_updates_filter_scopes_catch_errors_and_preserve_order() {
        let fixture = AutoUpdateTestImports {
            installed: serde_json::json!({"plugins":{
                "a@Market":[{"scope":"user"},{"scope":"local","projectPath":"/not-the-current-project"}],
                "b@market":[{"scope":"user"},{"scope":"managed"}],
                "c@other":[{"scope":"user"}],"empty@market":[],
                "already@market":[{"scope":"user"}],"error@market":[{"scope":"user"}],"unscoped":[{"scope":"user"}]
            }}),
            calls: Default::default(),
        };
        AUTOUPDATE_TEST_IMPORTS
            .scope(fixture.clone(), async {
                assert_eq!(
                    update_plugins_for_marketplaces(&HashSet::from(["market".into()])).await,
                    ["a@Market", "b@market"]
                );
                let calls = fixture.calls.lock().unwrap();
                assert_eq!(calls.len(), 5);
                assert!(
                    calls.iter().all(|(_, scope)| matches!(
                        scope,
                        PluginScope::User | PluginScope::Managed
                    ))
                );
                assert_eq!(
                    calls.last(),
                    Some(&("b@market".into(), PluginScope::Managed))
                );
            })
            .await;
    }
}
