//! Maps to: CC `bridge/bridgeStatusUtil.ts`.
//!
//! This module is intentionally pure. It does not start bridge transports,
//! subscribe to app state, write config, or contact claude.ai.

pub const CLAUDE_AI_BASE_URL: &str = "https://claude.ai";
pub const CLAUDE_AI_STAGING_BASE_URL: &str = "https://claude-ai.staging.ant.dev";
pub const CLAUDE_AI_LOCAL_BASE_URL: &str = "http://localhost:4000";

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `StatusState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusState {
    Idle,
    Attached,
    Titled,
    Reconnecting,
    Failed,
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `BridgeStatusInfo.color`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeStatusColor {
    Error,
    Warning,
    Success,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeStatusInfo {
    pub label: &'static str,
    pub color: BridgeStatusColor,
}

/// Maps to: CC `constants/product.ts` `isRemoteSessionStaging`.
pub fn is_remote_session_staging(session_id: Option<&str>, ingress_url: Option<&str>) -> bool {
    session_id
        .map(|session_id| session_id.contains("_staging_"))
        .unwrap_or(false)
        || ingress_url
            .map(|ingress_url| ingress_url.contains("staging"))
            .unwrap_or(false)
}

/// Maps to: CC `constants/product.ts` `isRemoteSessionLocal`.
pub fn is_remote_session_local(session_id: Option<&str>, ingress_url: Option<&str>) -> bool {
    session_id
        .map(|session_id| session_id.contains("_local_"))
        .unwrap_or(false)
        || ingress_url
            .map(|ingress_url| ingress_url.contains("localhost"))
            .unwrap_or(false)
}

/// Maps to: CC `constants/product.ts` `getClaudeAiBaseUrl`.
pub fn get_claude_ai_base_url(session_id: Option<&str>, ingress_url: Option<&str>) -> &'static str {
    if is_remote_session_local(session_id, ingress_url) {
        CLAUDE_AI_LOCAL_BASE_URL
    } else if is_remote_session_staging(session_id, ingress_url) {
        CLAUDE_AI_STAGING_BASE_URL
    } else {
        CLAUDE_AI_BASE_URL
    }
}

/// Maps to: CC `bridge/sessionIdCompat.ts` `toCompatSessionId` with the
/// official default gate state (enabled unless explicitly disabled).
pub fn to_compat_session_id(id: &str) -> String {
    id.strip_prefix("cse_")
        .map(|rest| format!("session_{rest}"))
        .unwrap_or_else(|| id.to_string())
}

/// Maps to: CC `constants/product.ts` `getRemoteSessionUrl`.
pub fn get_remote_session_url(session_id: &str, ingress_url: Option<&str>) -> String {
    let compat_id = to_compat_session_id(session_id);
    let base_url = get_claude_ai_base_url(Some(&compat_id), ingress_url);
    format!("{base_url}/code/{compat_id}")
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `buildBridgeConnectUrl`.
pub fn build_bridge_connect_url(environment_id: &str, ingress_url: Option<&str>) -> String {
    let base_url = get_claude_ai_base_url(None, ingress_url);
    format!("{base_url}/code?bridge={environment_id}")
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `buildBridgeSessionUrl`.
pub fn build_bridge_session_url(
    session_id: &str,
    environment_id: &str,
    ingress_url: Option<&str>,
) -> String {
    format!(
        "{}?bridge={environment_id}",
        get_remote_session_url(session_id, ingress_url)
    )
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `getBridgeStatus`.
pub fn bridge_error_present(error: Option<&str>) -> bool {
    error.map(|value| !value.is_empty()).unwrap_or(false)
}

pub fn get_bridge_status(
    error: Option<&str>,
    connected: bool,
    session_active: bool,
    reconnecting: bool,
) -> BridgeStatusInfo {
    if bridge_error_present(error) {
        BridgeStatusInfo {
            label: "Remote Control failed",
            color: BridgeStatusColor::Error,
        }
    } else if reconnecting {
        BridgeStatusInfo {
            label: "Remote Control reconnecting",
            color: BridgeStatusColor::Warning,
        }
    } else if session_active || connected {
        BridgeStatusInfo {
            label: "Remote Control active",
            color: BridgeStatusColor::Success,
        }
    } else {
        BridgeStatusInfo {
            label: "Remote Control connecting…",
            color: BridgeStatusColor::Warning,
        }
    }
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `buildIdleFooterText`.
pub fn build_idle_footer_text(url: &str) -> String {
    format!("Code everywhere with the Claude app or {url}")
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `buildActiveFooterText`.
pub fn build_active_footer_text(url: &str) -> String {
    format!("Continue coding in the Claude app or {url}")
}

/// Maps to: CC `bridge/bridgeStatusUtil.ts` `FAILED_FOOTER_TEXT`.
pub const FAILED_FOOTER_TEXT: &str = "Something went wrong, please try again";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_status_matches_official_state_machine() {
        assert_eq!(
            get_bridge_status(Some("boom"), false, false, false),
            BridgeStatusInfo {
                label: "Remote Control failed",
                color: BridgeStatusColor::Error,
            }
        );
        assert_eq!(
            get_bridge_status(Some(""), false, false, false).label,
            "Remote Control connecting…"
        );
        assert_eq!(
            get_bridge_status(None, false, false, true),
            BridgeStatusInfo {
                label: "Remote Control reconnecting",
                color: BridgeStatusColor::Warning,
            }
        );
        assert_eq!(
            get_bridge_status(None, true, false, false).label,
            "Remote Control active"
        );
        assert_eq!(
            get_bridge_status(None, false, true, false).label,
            "Remote Control active"
        );
        assert_eq!(
            get_bridge_status(None, false, false, false).label,
            "Remote Control connecting…"
        );
    }

    #[test]
    fn bridge_urls_match_official_product_helpers() {
        assert_eq!(
            build_bridge_connect_url("env_123", None),
            "https://claude.ai/code?bridge=env_123"
        );
        assert_eq!(
            build_bridge_connect_url("env_123", Some("https://staging.example")),
            "https://claude-ai.staging.ant.dev/code?bridge=env_123"
        );
        assert_eq!(
            build_bridge_connect_url("env_123", Some("http://localhost:3000")),
            "http://localhost:4000/code?bridge=env_123"
        );
        assert_eq!(
            build_bridge_session_url("cse_abc", "env_123", None),
            "https://claude.ai/code/session_abc?bridge=env_123"
        );
        assert_eq!(
            build_bridge_session_url("session_local_abc", "env_123", None),
            "http://localhost:4000/code/session_local_abc?bridge=env_123"
        );
    }

    #[test]
    fn bridge_footer_text_matches_official_copy() {
        assert_eq!(
            build_idle_footer_text("https://claude.ai/code?bridge=env"),
            "Code everywhere with the Claude app or https://claude.ai/code?bridge=env"
        );
        assert_eq!(
            build_active_footer_text("https://claude.ai/code/session?bridge=env"),
            "Continue coding in the Claude app or https://claude.ai/code/session?bridge=env"
        );
        assert_eq!(FAILED_FOOTER_TEXT, "Something went wrong, please try again");
    }
}
