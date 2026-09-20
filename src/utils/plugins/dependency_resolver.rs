//! Maps to: CC `utils/plugins/dependencyResolver.ts`.
use super::plugin_identifier::parse_plugin_identifier;
use crate::types::plugin::{LoadedPlugin, PluginDependencyReason, PluginError};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

const INLINE_MARKETPLACE: &str = "inline";
/// Maps to: CC dependencyResolver.ts:35-43#qualifyDependency.
pub fn qualify_dependency(dependency: &str, declaring_plugin_id: &str) -> String {
    if parse_plugin_identifier(dependency)
        .marketplace
        .as_ref()
        .is_some_and(|m| !m.is_empty())
    {
        return dependency.into();
    }
    match parse_plugin_identifier(declaring_plugin_id)
        .marketplace
        .filter(|m| !m.is_empty() && m != INLINE_MARKETPLACE)
    {
        Some(marketplace) => format!("{dependency}@{marketplace}"),
        None => dependency.into(),
    }
}
/// Maps to: CC dependencyResolver.ts:53-64#ResolutionResult.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolutionResult {
    Success {
        closure: Vec<String>,
    },
    Cycle {
        chain: Vec<String>,
    },
    NotFound {
        missing: String,
        required_by: String,
    },
    CrossMarketplace {
        dependency: String,
        required_by: String,
    },
}
/// Maps to: CC dependencyResolver.ts:93-153#resolveDependencyClosure.
pub async fn resolve_dependency_closure<F, Fut>(
    root_id: &str,
    lookup: F,
    already_enabled: &HashSet<String>,
    allowed_cross_marketplaces: &HashSet<String>,
) -> anyhow::Result<ResolutionResult>
where
    F: Fn(String) -> Fut + Sync,
    Fut: std::future::Future<Output = anyhow::Result<Option<Value>>> + Send,
{
    let root_marketplace = parse_plugin_identifier(root_id).marketplace;
    let mut closure = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    // Source nested walk stays nested; boxed recursion is the async Future size carrier.
    fn walk<'a, F, Fut>(
        id: String,
        required_by: String,
        root_id: &'a str,
        root_marketplace: &'a Option<String>,
        lookup: &'a F,
        already_enabled: &'a HashSet<String>,
        allowed: &'a HashSet<String>,
        closure: &'a mut Vec<String>,
        visited: &'a mut HashSet<String>,
        stack: &'a mut Vec<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<Option<ResolutionResult>>> + Send + 'a>,
    >
    where
        F: Fn(String) -> Fut + Sync + 'a,
        Fut: std::future::Future<Output = anyhow::Result<Option<Value>>> + Send + 'a,
    {
        Box::pin(async move {
            if id != root_id && already_enabled.contains(&id) {
                return Ok(None);
            }
            let marketplace = parse_plugin_identifier(&id).marketplace;
            if marketplace != *root_marketplace
                && !marketplace
                    .as_ref()
                    .is_some_and(|m| !m.is_empty() && allowed.contains(m))
            {
                return Ok(Some(ResolutionResult::CrossMarketplace {
                    dependency: id,
                    required_by,
                }));
            }
            if stack.contains(&id) {
                let mut chain = stack.clone();
                chain.push(id);
                return Ok(Some(ResolutionResult::Cycle { chain }));
            }
            if !visited.insert(id.clone()) {
                return Ok(None);
            }
            let Some(entry) = lookup(id.clone()).await? else {
                return Ok(Some(ResolutionResult::NotFound {
                    missing: id,
                    required_by,
                }));
            };
            stack.push(id.clone());
            for raw in entry
                .get("dependencies")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let dependency = qualify_dependency(raw, &id);
                if let Some(error) = walk(
                    dependency,
                    id.clone(),
                    root_id,
                    root_marketplace,
                    lookup,
                    already_enabled,
                    allowed,
                    closure,
                    visited,
                    stack,
                )
                .await?
                {
                    return Ok(Some(error));
                }
            }
            stack.pop();
            closure.push(id);
            Ok(None)
        })
    }
    if let Some(error) = walk(
        root_id.into(),
        root_id.into(),
        root_id,
        &root_marketplace,
        &lookup,
        already_enabled,
        allowed_cross_marketplaces,
        &mut closure,
        &mut visited,
        &mut stack,
    )
    .await?
    {
        return Ok(error);
    }
    Ok(ResolutionResult::Success { closure })
}
/// Maps to: CC dependencyResolver.ts:171-233#verifyAndDemote.
pub fn verify_and_demote(plugins: &[LoadedPlugin]) -> (HashSet<String>, Vec<PluginError>) {
    let known: HashSet<_> = plugins.iter().map(|p| p.source.clone()).collect();
    let mut enabled: HashSet<_> = plugins
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.source.clone())
        .collect();
    let known_by_name: HashSet<_> = plugins
        .iter()
        .map(|p| parse_plugin_identifier(&p.source).name)
        .collect();
    let mut enabled_by_name: HashMap<String, usize> = HashMap::new();
    for id in &enabled {
        *enabled_by_name
            .entry(parse_plugin_identifier(id).name)
            .or_default() += 1;
    }
    let mut errors = Vec::new();
    let mut changed = true;
    while changed {
        changed = false;
        for plugin in plugins {
            if !enabled.contains(&plugin.source) {
                continue;
            }
            for raw in &plugin.manifest.dependencies {
                let dep = qualify_dependency(raw, &plugin.source);
                let is_bare = parse_plugin_identifier(&dep)
                    .marketplace
                    .is_none_or(|m| m.is_empty());
                let satisfied = if is_bare {
                    enabled_by_name.get(&dep).copied().unwrap_or(0) > 0
                } else {
                    enabled.contains(&dep)
                };
                if !satisfied {
                    enabled.remove(&plugin.source);
                    let count = enabled_by_name.get(&plugin.name).copied().unwrap_or(0);
                    if count <= 1 {
                        enabled_by_name.remove(&plugin.name);
                    } else {
                        enabled_by_name.insert(plugin.name.clone(), count - 1);
                    }
                    let is_known = if is_bare {
                        known_by_name.contains(&dep)
                    } else {
                        known.contains(&dep)
                    };
                    errors.push(PluginError::DependencyUnsatisfied {
                        source: plugin.source.clone(),
                        plugin: plugin.name.clone(),
                        dependency: dep,
                        reason: if is_known {
                            PluginDependencyReason::NotEnabled
                        } else {
                            PluginDependencyReason::NotFound
                        },
                    });
                    changed = true;
                    break;
                }
            }
        }
    }
    (
        plugins
            .iter()
            .filter(|p| p.enabled && !enabled.contains(&p.source))
            .map(|p| p.source.clone())
            .collect(),
        errors,
    )
}
/// Maps to: CC dependencyResolver.ts:244-262#findReverseDependents.
pub fn find_reverse_dependents(plugin_id: &str, plugins: &[LoadedPlugin]) -> Vec<String> {
    let name = parse_plugin_identifier(plugin_id).name;
    plugins
        .iter()
        .filter(|p| {
            p.enabled
                && p.source != plugin_id
                && p.manifest.dependencies.iter().any(|dep| {
                    let qualified = qualify_dependency(dep, &p.source);
                    if parse_plugin_identifier(&qualified)
                        .marketplace
                        .is_some_and(|m| !m.is_empty())
                    {
                        qualified == plugin_id
                    } else {
                        qualified == name
                    }
                })
        })
        .map(|p| p.name.clone())
        .collect()
}
/// Maps to: CC dependencyResolver.ts:276-284#getEnabledPluginIdsForScope.
pub fn get_enabled_plugin_ids_for_scope(
    source: crate::utils::settings::constants::SettingSource,
) -> HashSet<String> {
    crate::utils::settings::get_settings_for_source(source)
        .and_then(|s| s.enabled_plugins)
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, v)| v == &Value::Bool(true) || v.is_array())
        .map(|(id, _)| id)
        .collect()
}
/// Maps to: CC dependencyResolver.ts:290-295#formatDependencyCountSuffix.
pub fn format_dependency_count_suffix(deps: &[String]) -> String {
    if deps.is_empty() {
        String::new()
    } else {
        format!(
            " (+ {} {})",
            deps.len(),
            if deps.len() == 1 {
                "dependency"
            } else {
                "dependencies"
            }
        )
    }
}
/// Maps to: CC dependencyResolver.ts:302-305#formatReverseDependentsSuffix.
pub fn format_reverse_dependents_suffix(deps: Option<&[String]>) -> String {
    deps.filter(|d| !d.is_empty())
        .map(|d| format!(" — warning: required by {}", d.join(", ")))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn dependency_closure_matches_official_root_reinstall_and_cross_marketplace_order() {
        // Source :111-119 skips existing deps before policy, never skips root.
        let enabled = HashSet::from(["root@a".into(), "other@b".into()]);
        let result = resolve_dependency_closure(
            "root@a",
            |id| async move {
                Ok((id == "root@a").then(|| serde_json::json!({"dependencies":["other@b"]})))
            },
            &enabled,
            &HashSet::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            result,
            ResolutionResult::Success {
                closure: vec!["root@a".into()]
            }
        );
        let result = resolve_dependency_closure(
            "root@a",
            |_| async { Ok(Some(serde_json::json!({"dependencies":["other@b"]}))) },
            &HashSet::new(),
            &HashSet::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            result,
            ResolutionResult::CrossMarketplace {
                dependency: "other@b".into(),
                required_by: "root@a".into()
            }
        );
    }
}
