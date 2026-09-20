//! Maps to: CC `utils/model/antModels.ts`.
//! L1: `Compile-time distribution capability projection` (PORTING.md).

use crate::utils::build_profile::{InternalCapability, has_internal_capability};
use serde::{Deserialize, Serialize};

/// Maps to: CC `utils/model/antModels.ts:4-16` `AntModel`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntModel {
    pub alias: String,
    pub model: String,
    pub label: String,
    pub description: Option<String>,
    pub default_effort_value: Option<f64>,
    pub default_effort_level: Option<String>,
    pub context_window: Option<u64>,
    pub default_max_tokens: Option<u64>,
    pub upper_max_tokens_limit: Option<u64>,
    pub always_on_thinking: Option<bool>,
}

/// Maps to: CC `utils/model/antModels.ts:18-22` `AntModelSwitchCalloutConfig`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntModelSwitchCalloutConfig {
    pub model_alias: Option<String>,
    pub description: String,
    pub version: String,
}

/// Maps to: CC `utils/model/antModels.ts:24-30` `AntModelOverrideConfig`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntModelOverrideConfig {
    pub default_model: Option<String>,
    pub default_model_effort_level: Option<String>,
    pub default_system_prompt_suffix: Option<String>,
    pub ant_models: Option<Vec<AntModel>>,
    pub switch_callout: Option<AntModelSwitchCalloutConfig>,
}

/// Maps to: CC `utils/model/antModels.ts:34-43` `getAntModelOverrideConfig`.
pub fn get_ant_model_override_config() -> Option<AntModelOverrideConfig> {
    if !has_internal_capability(InternalCapability::Models) {
        return None;
    }
    crate::services::analytics::growthbook::get_feature_value_cached_may_be_stale(
        "tengu_ant_model_override",
        None,
    )
}

/// Maps to: CC `utils/model/antModels.ts:45-50` `getAntModels`.
pub fn get_ant_models() -> Vec<AntModel> {
    if !has_internal_capability(InternalCapability::Models) {
        return Vec::new();
    }
    get_ant_model_override_config()
        .and_then(|config| config.ant_models)
        .unwrap_or_default()
}

/// Maps to: CC `utils/model/antModels.ts:52-66` `resolveAntModel`.
pub fn resolve_ant_model(model: Option<&str>) -> Option<AntModel> {
    if !has_internal_capability(InternalCapability::Models) {
        return None;
    }
    let model = model?;
    let lower = model.to_lowercase();
    get_ant_models().into_iter().find(|candidate| {
        candidate.alias == model || lower.contains(&candidate.model.to_lowercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_model_override_gate_matches_official() {
        if !has_internal_capability(InternalCapability::Models) {
            assert_eq!(get_ant_model_override_config(), None);
            assert!(get_ant_models().is_empty());
            assert_eq!(resolve_ant_model(Some("any-model")), None);
        }
        assert_eq!(resolve_ant_model(None), None);
    }
}
