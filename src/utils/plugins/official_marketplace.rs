//! Maps to: CC `utils/plugins/officialMarketplace.ts`.

/// Maps to: CC `utils/plugins/officialMarketplace.ts:15-18#OFFICIAL_MARKETPLACE_SOURCE`.
/// Lazy immutable JSON represents the source's object-valued constant.
pub static OFFICIAL_MARKETPLACE_SOURCE: std::sync::LazyLock<serde_json::Value> =
    std::sync::LazyLock::new(|| {
        serde_json::json!({
            "source": "github",
            "repo": "anthropics/claude-plugins-official"
        })
    });

/// Maps to: CC `utils/plugins/officialMarketplace.ts:25#OFFICIAL_MARKETPLACE_NAME`.
pub const OFFICIAL_MARKETPLACE_NAME: &str = "claude-plugins-official";
