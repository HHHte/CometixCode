//! Provider-specific model ID tables.
//! Maps to CC `utils/model/configs.ts`.

use crate::utils::model::providers::ApiProvider;

/// Maps to: CC `utils/model/configs.ts:4` `ModelConfig`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelConfig {
    pub first_party: &'static str,
    pub bedrock: &'static str,
    pub vertex: &'static str,
    pub foundry: &'static str,
}

impl ModelConfig {
    pub fn for_provider(&self, provider: ApiProvider) -> &'static str {
        match provider {
            ApiProvider::FirstParty => self.first_party,
            ApiProvider::Bedrock => self.bedrock,
            ApiProvider::Vertex => self.vertex,
            ApiProvider::Foundry => self.foundry,
        }
    }
}

/// Maps to: CC `utils/model/configs.ts:9-14` `CLAUDE_3_7_SONNET_CONFIG`.
pub const CLAUDE_3_7_SONNET_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-3-7-sonnet-20250219",
    bedrock: "us.anthropic.claude-3-7-sonnet-20250219-v1:0",
    vertex: "claude-3-7-sonnet@20250219",
    foundry: "claude-3-7-sonnet",
};

/// Maps to: CC `utils/model/configs.ts:16-21` `CLAUDE_3_5_V2_SONNET_CONFIG`.
pub const CLAUDE_3_5_V2_SONNET_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-3-5-sonnet-20241022",
    bedrock: "anthropic.claude-3-5-sonnet-20241022-v2:0",
    vertex: "claude-3-5-sonnet-v2@20241022",
    foundry: "claude-3-5-sonnet",
};

/// Maps to: CC `utils/model/configs.ts:23-28` `CLAUDE_3_5_HAIKU_CONFIG`.
pub const CLAUDE_3_5_HAIKU_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-3-5-haiku-20241022",
    bedrock: "us.anthropic.claude-3-5-haiku-20241022-v1:0",
    vertex: "claude-3-5-haiku@20241022",
    foundry: "claude-3-5-haiku",
};

/// Maps to: CC `utils/model/configs.ts:30-35` `CLAUDE_HAIKU_4_5_CONFIG`.
pub const CLAUDE_HAIKU_4_5_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-haiku-4-5-20251001",
    bedrock: "us.anthropic.claude-haiku-4-5-20251001-v1:0",
    vertex: "claude-haiku-4-5@20251001",
    foundry: "claude-haiku-4-5",
};

/// Maps to: CC `utils/model/configs.ts:37-42` `CLAUDE_SONNET_4_CONFIG`.
pub const CLAUDE_SONNET_4_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-sonnet-4-20250514",
    bedrock: "us.anthropic.claude-sonnet-4-20250514-v1:0",
    vertex: "claude-sonnet-4@20250514",
    foundry: "claude-sonnet-4",
};

/// Maps to: CC `utils/model/configs.ts:44-49` `CLAUDE_SONNET_4_5_CONFIG`.
pub const CLAUDE_SONNET_4_5_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-sonnet-4-5-20250929",
    bedrock: "us.anthropic.claude-sonnet-4-5-20250929-v1:0",
    vertex: "claude-sonnet-4-5@20250929",
    foundry: "claude-sonnet-4-5",
};

/// Maps to: CC `utils/model/configs.ts:51-56` `CLAUDE_OPUS_4_CONFIG`.
pub const CLAUDE_OPUS_4_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-opus-4-20250514",
    bedrock: "us.anthropic.claude-opus-4-20250514-v1:0",
    vertex: "claude-opus-4@20250514",
    foundry: "claude-opus-4",
};

/// Maps to: CC `utils/model/configs.ts:58-63` `CLAUDE_OPUS_4_1_CONFIG`.
pub const CLAUDE_OPUS_4_1_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-opus-4-1-20250805",
    bedrock: "us.anthropic.claude-opus-4-1-20250805-v1:0",
    vertex: "claude-opus-4-1@20250805",
    foundry: "claude-opus-4-1",
};

/// Maps to: CC `utils/model/configs.ts:65-70` `CLAUDE_OPUS_4_5_CONFIG`.
pub const CLAUDE_OPUS_4_5_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-opus-4-5-20251101",
    bedrock: "us.anthropic.claude-opus-4-5-20251101-v1:0",
    vertex: "claude-opus-4-5@20251101",
    foundry: "claude-opus-4-5",
};

/// Maps to: CC `utils/model/configs.ts:72-77` `CLAUDE_OPUS_4_6_CONFIG`.
pub const CLAUDE_OPUS_4_6_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-opus-4-6",
    bedrock: "us.anthropic.claude-opus-4-6-v1",
    vertex: "claude-opus-4-6",
    foundry: "claude-opus-4-6",
};

/// Maps to: CC `utils/model/configs.ts:79-84` `CLAUDE_SONNET_4_6_CONFIG`.
pub const CLAUDE_SONNET_4_6_CONFIG: ModelConfig = ModelConfig {
    first_party: "claude-sonnet-4-6",
    bedrock: "us.anthropic.claude-sonnet-4-6",
    vertex: "claude-sonnet-4-6",
    foundry: "claude-sonnet-4-6",
};

/// Maps to: CC `utils/model/configs.ts:101` `ModelKey`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelKey {
    Haiku35,
    Haiku45,
    Sonnet35,
    Sonnet37,
    Sonnet40,
    Sonnet45,
    Sonnet46,
    Opus40,
    Opus41,
    Opus45,
    Opus46,
}

/// Maps to: CC `utils/model/configs.ts:87-99` `ALL_MODEL_CONFIGS`, in the same
/// declaration order — `modelStrings.ts:23` derives `MODEL_KEYS` from it.
pub const ALL_MODEL_CONFIGS: &[(ModelKey, ModelConfig)] = &[
    (ModelKey::Haiku35, CLAUDE_3_5_HAIKU_CONFIG),
    (ModelKey::Haiku45, CLAUDE_HAIKU_4_5_CONFIG),
    (ModelKey::Sonnet35, CLAUDE_3_5_V2_SONNET_CONFIG),
    (ModelKey::Sonnet37, CLAUDE_3_7_SONNET_CONFIG),
    (ModelKey::Sonnet40, CLAUDE_SONNET_4_CONFIG),
    (ModelKey::Sonnet45, CLAUDE_SONNET_4_5_CONFIG),
    (ModelKey::Sonnet46, CLAUDE_SONNET_4_6_CONFIG),
    (ModelKey::Opus40, CLAUDE_OPUS_4_CONFIG),
    (ModelKey::Opus41, CLAUDE_OPUS_4_1_CONFIG),
    (ModelKey::Opus45, CLAUDE_OPUS_4_5_CONFIG),
    (ModelKey::Opus46, CLAUDE_OPUS_4_6_CONFIG),
];

/// Maps to: CC `utils/model/configs.ts:113-118` `CANONICAL_ID_TO_KEY`.
pub fn canonical_id_to_key(canonical_id: &str) -> Option<ModelKey> {
    ALL_MODEL_CONFIGS
        .iter()
        .find_map(|(key, config)| (config.first_party == canonical_id).then_some(*key))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Maps to: CC `utils/model/configs.ts:108-110` `CANONICAL_MODEL_IDS`, whose
    /// order the `modelStrings.ts` table build depends on.
    #[test]
    fn all_model_configs_match_the_official_registration_order_and_ids() {
        let canonical_ids: Vec<&str> = ALL_MODEL_CONFIGS
            .iter()
            .map(|(_, config)| config.first_party)
            .collect();
        assert_eq!(
            canonical_ids,
            [
                "claude-3-5-haiku-20241022",
                "claude-haiku-4-5-20251001",
                "claude-3-5-sonnet-20241022",
                "claude-3-7-sonnet-20250219",
                "claude-sonnet-4-20250514",
                "claude-sonnet-4-5-20250929",
                "claude-sonnet-4-6",
                "claude-opus-4-20250514",
                "claude-opus-4-1-20250805",
                "claude-opus-4-5-20251101",
                "claude-opus-4-6",
            ]
        );
    }

    #[test]
    fn provider_lookup_selects_the_official_per_provider_ids() {
        assert_eq!(
            CLAUDE_OPUS_4_6_CONFIG.for_provider(ApiProvider::Bedrock),
            "us.anthropic.claude-opus-4-6-v1"
        );
        assert_eq!(
            CLAUDE_3_5_V2_SONNET_CONFIG.for_provider(ApiProvider::Vertex),
            "claude-3-5-sonnet-v2@20241022"
        );
        assert_eq!(
            CLAUDE_HAIKU_4_5_CONFIG.for_provider(ApiProvider::Foundry),
            "claude-haiku-4-5"
        );
    }

    #[test]
    fn canonical_ids_map_back_to_their_official_keys() {
        assert_eq!(
            canonical_id_to_key("claude-opus-4-6"),
            Some(ModelKey::Opus46)
        );
        assert_eq!(
            canonical_id_to_key("claude-sonnet-4-5-20250929"),
            Some(ModelKey::Sonnet45)
        );
        assert_eq!(canonical_id_to_key("claude-opus-4-6-v1"), None);
    }
}
