//! Maps to: CC `utils/plugins/fetchTelemetry.ts`.
//! Assembles events for the existing analytics queue; no network sink here.

use super::official_marketplace::OFFICIAL_MARKETPLACE_NAME;

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:21-27#PluginFetchSource`.
#[derive(Clone, Copy)]
pub enum PluginFetchSource {
    InstallCounts,
    MarketplaceClone,
    MarketplacePull,
    MarketplaceUrl,
    PluginClone,
    Mcpb,
}

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:29#PluginFetchOutcome`.
#[derive(Clone, Copy)]
pub enum PluginFetchOutcome {
    Success,
    Failure,
    CacheHit,
}

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:35-46#KNOWN_PUBLIC_HOSTS`.
const KNOWN_PUBLIC_HOSTS: &[&str] = &[
    "github.com",
    "raw.githubusercontent.com",
    "objects.githubusercontent.com",
    "gist.githubusercontent.com",
    "gitlab.com",
    "bitbucket.org",
    "codeberg.org",
    "dev.azure.com",
    "ssh.dev.azure.com",
    "storage.googleapis.com",
];

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:54-68#extractHost`.
fn extract_host(url_or_spec: &str) -> String {
    let scp = regress::Regex::new(r"^[^@/]+@([^:/]+):").expect("source regex is valid");
    let host = if let Some(capture) = scp.find(url_or_spec) {
        url_or_spec[capture.group(1).unwrap()].to_owned()
    } else {
        match url::Url::parse(url_or_spec) {
            Ok(url) => url.host_str().unwrap_or("").to_owned(),
            Err(_) => return "unknown".into(),
        }
    };
    let normalized = host.to_lowercase();
    if KNOWN_PUBLIC_HOSTS.contains(&normalized.as_str()) {
        normalized
    } else {
        "other".into()
    }
}

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:75-77#isOfficialRepo`.
fn is_official_repo(url_or_spec: &str) -> bool {
    url_or_spec.contains(&format!("anthropics/{OFFICIAL_MARKETPLACE_NAME}"))
}

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:79-97#logPluginFetch`.
pub fn log_plugin_fetch(
    source: PluginFetchSource,
    url_or_spec: Option<&str>,
    outcome: PluginFetchOutcome,
    duration_ms: f64,
    error_kind: Option<&str>,
) {
    let source = match source {
        PluginFetchSource::InstallCounts => "install_counts",
        PluginFetchSource::MarketplaceClone => "marketplace_clone",
        PluginFetchSource::MarketplacePull => "marketplace_pull",
        PluginFetchSource::MarketplaceUrl => "marketplace_url",
        PluginFetchSource::PluginClone => "plugin_clone",
        PluginFetchSource::Mcpb => "mcpb",
    };
    let outcome = match outcome {
        PluginFetchOutcome::Success => "success",
        PluginFetchOutcome::Failure => "failure",
        PluginFetchOutcome::CacheHit => "cache_hit",
    };
    let url = url_or_spec.filter(|url| !url.is_empty());
    // JavaScript Math.round ties toward +infinity and preserves negative zero.
    let floor = duration_ms.floor();
    let rounded = if duration_ms - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    };
    let rounded = if rounded == 0.0 {
        0.0_f64.copysign(duration_ms)
    } else {
        rounded
    };
    let mut metadata = serde_json::json!({
        "source": source,
        "host": url.map(extract_host).unwrap_or_else(|| "unknown".into()),
        "is_official": url.is_some_and(is_official_repo),
        "outcome": outcome,
        "duration_ms": rounded,
    });
    if let Some(kind) = error_kind.filter(|kind| !kind.is_empty()) {
        metadata["error_kind"] = kind.into();
    }
    crate::services::analytics::log_event("tengu_plugin_remote_fetch", metadata);
}

/// Maps to: CC `utils/plugins/fetchTelemetry.ts:108-136#classifyFetchError`.
/// Native Error::Display / git stderr carries source `error.message ?? error`.
/// Arbitrary non-error JS objects are outside this native string boundary.
pub fn classify_fetch_error(error: &(impl std::fmt::Display + ?Sized)) -> &'static str {
    let msg = error.to_string();
    for (pattern, kind) in [
        (
            "ENOTFOUND|ECONNREFUSED|EAI_AGAIN|Could not resolve host|Connection refused",
            "dns_or_refused",
        ),
        ("ETIMEDOUT|timed out|timeout", "timeout"),
        (
            "ECONNRESET|socket hang up|Connection reset by peer|remote end hung up",
            "conn_reset",
        ),
        ("403|401|authentication|permission denied", "auth"),
        ("404|not found|repository not found", "not_found"),
        ("certificate|SSL|TLS|unable to get local issuer", "tls"),
        (
            "Invalid response format|Invalid marketplace schema",
            "invalid_schema",
        ),
    ] {
        if regress::Regex::with_flags(pattern, "i")
            .expect("source regex is valid")
            .find(&msg)
            .is_some()
        {
            return kind;
        }
    }
    "other"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_telemetry_matches_official_hosts_and_error_precedence() {
        // CC utils/plugins/fetchTelemetry.ts:54-77,108-136; original Bun
        // oracle: research/proof/plugin-discover-counts-0914/oracle.json.
        for (spec, expected) in [
            (
                "git@GitHub.COM:anthropics/claude-plugins-official.git",
                "github.com",
            ),
            ("https://my.private/path", "other"),
            ("/tmp/local", "unknown"),
            ("file:///tmp/local", "other"),
            ("mailto:user@github.com", "other"),
            ("git+ssh://git@github.com/repo", "github.com"),
            ("https://github.com./a", "other"),
            ("C:\\local", "other"),
        ] {
            assert_eq!(extract_host(spec), expected, "{spec}");
        }
        for (input, expected) in [
            ("ENOTFOUND timeout", "dns_or_refused"),
            ("EAI_AGAIN", "dns_or_refused"),
            ("Could not resolve host timeout", "dns_or_refused"),
            ("ETIMEDOUT", "timeout"),
            ("Connection reset by peer 403", "conn_reset"),
            ("404 certificate", "not_found"),
            ("authentication", "auth"),
            ("SSL", "tls"),
            ("Invalid response format", "invalid_schema"),
            ("foo", "other"),
        ] {
            assert_eq!(classify_fetch_error(input), expected);
        }
        assert!(is_official_repo(
            "https://GITHUB.com/anthropics/claude-plugins-official-extra"
        ));
        assert!(!is_official_repo(
            "https://github.com/Anthropics/claude-plugins-official"
        ));
    }

    #[test]
    fn log_plugin_fetch_matches_official_existing_queue_fields() {
        // CC utils/plugins/fetchTelemetry.ts:79-97 builds metadata;
        // services/analytics/index.ts:138-151 queues it when no sink is attached.
        log_plugin_fetch(
            PluginFetchSource::InstallCounts,
            Some("https://private.invalid/unique-counts-fixture"),
            PluginFetchOutcome::Failure,
            1.5,
            Some("invalid_schema"),
        );
        assert!(crate::services::analytics::queued_events_for_test().iter().any(|(name, value)| name == "tengu_plugin_remote_fetch" && *value == serde_json::json!({"source":"install_counts","host":"other","is_official":false,"outcome":"failure","duration_ms":2.0,"error_kind":"invalid_schema"})));
        log_plugin_fetch(
            PluginFetchSource::Mcpb,
            None,
            PluginFetchOutcome::CacheHit,
            -0.5,
            Some(""),
        );
        assert!(
            crate::services::analytics::queued_events_for_test()
                .iter()
                .any(|(name, value)| name == "tengu_plugin_remote_fetch"
                    && value["source"] == "mcpb"
                    && value["duration_ms"]
                        .as_f64()
                        .is_some_and(|n| n == 0.0 && n.is_sign_negative())
                    && value.get("error_kind").is_none())
        );
    }
}
