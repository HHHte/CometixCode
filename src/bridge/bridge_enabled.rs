//! Maps to: CC `bridge/bridgeEnabled.ts` — the runtime entitlement check
//! subset. The blocking/diagnostic variants (`isBridgeEnabledBlocking`,
//! `getBridgeDisabledReason`) and the v2/env-less/CCR-mirror gates stay with
//! the unported bridge runtime (see MODULE_MAP `bridge/*` rows).

use crate::utils::auth::is_claude_ai_subscriber;
use crate::utils::feature_flags::{FeatureFlag, feature_enabled};

/// Maps to: CC `bridge/bridgeEnabled.ts:28-36` `isBridgeEnabled`.
///
/// Remote Control requires a claude.ai subscription (the bridge auths to CCR
/// with the claude.ai OAuth token); `isClaudeAISubscriber()` excludes
/// Bedrock/Vertex/Foundry, apiKeyHelper/gateway deployments, env-var API
/// keys, and Console API logins. CC's positive `feature('BRIDGE_MODE')`
/// ternary is mirrored by `FeatureFlag::BridgeMode` (production default:
/// enabled, matching CC scripts/build.ts:41). CC wraps its subscriber call
/// in try/catch for pre-config access; the Rust config path is non-throwing,
/// so no equivalent guard is needed.
///
/// Production note: the per-account `tengu_ccr_bridge` entitlement is
/// unresolvable without the GrowthBook network runtime, so it is a
/// source-controlled switch (`FeatureFlag::RemoteControlEntitlement`, default
/// `false`) rather than a read of the shared `cachedGrowthBookFeatures` config
/// that official CC populates on the same machine.
pub fn is_bridge_enabled() -> bool {
    if feature_enabled(FeatureFlag::BridgeMode) {
        is_claude_ai_subscriber() && feature_enabled(FeatureFlag::RemoteControlEntitlement)
    } else {
        false
    }
}
