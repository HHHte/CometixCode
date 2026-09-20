//! Maps to: CC `utils/settings/pluginOnlyPolicy.ts`.
//!
//! Source-aware strict plugin-only policy helpers. Managed policy can lock a
//! customization surface to plugin/admin-controlled sources; user/project/local
//! sources are then skipped at the caller boundary that knows the source.

/// Maps to: CC `CustomizationSurface`.
pub type CustomizationSurface = &'static str;

/// Maps to: CC `isRestrictedToPluginOnly(surface)` policy predicate with an
/// explicit policy value for deterministic tests and source-specific callers.
pub fn is_restricted_to_plugin_only_with_policy(
    surface: &str,
    policy: Option<&serde_json::Value>,
) -> bool {
    match policy {
        Some(serde_json::Value::Bool(true)) => true,
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .any(|item| item.as_str().is_some_and(|item| item == surface)),
        _ => false,
    }
}

/// Maps to: CC `utils/settings/pluginOnlyPolicy.ts#isRestrictedToPluginOnly`.
pub fn is_restricted_to_plugin_only(surface: &str) -> bool {
    let policy = crate::utils::settings::get_settings_for_source(
        crate::utils::settings::constants::SettingSource::Policy,
    )
    .and_then(|settings| settings.strict_plugin_only_customization);
    is_restricted_to_plugin_only_with_policy(surface, policy.as_ref())
}

/// Maps to: CC `utils/settings/pluginOnlyPolicy.ts#isSourceAdminTrusted`.
pub fn is_source_admin_trusted(source: &str) -> bool {
    matches!(
        source,
        "plugin" | "policySettings" | "built-in" | "builtin" | "bundled"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_plugin_only_policy_matches_official_bool_and_array_forms() {
        assert!(!is_restricted_to_plugin_only_with_policy("mcp", None));
        assert!(is_restricted_to_plugin_only_with_policy(
            "mcp",
            Some(&serde_json::json!(true))
        ));
        assert!(is_restricted_to_plugin_only_with_policy(
            "mcp",
            Some(&serde_json::json!(["hooks", "mcp"]))
        ));
        assert!(!is_restricted_to_plugin_only_with_policy(
            "agents",
            Some(&serde_json::json!(["hooks", "mcp"]))
        ));
    }

    #[test]
    fn admin_trusted_sources_match_official_set() {
        assert!(is_source_admin_trusted("plugin"));
        assert!(is_source_admin_trusted("policySettings"));
        assert!(is_source_admin_trusted("built-in"));
        assert!(is_source_admin_trusted("builtin"));
        assert!(is_source_admin_trusted("bundled"));
        assert!(!is_source_admin_trusted("projectSettings"));
        assert!(!is_source_admin_trusted("userSettings"));
    }
}
