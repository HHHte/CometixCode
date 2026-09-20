//! Shared LSP service types.
//!
//! Maps to:
//! - CC `services/lsp/types.ts`
//! - CC `services/diagnosticTracking.ts` `Diagnostic` / `DiagnosticFile`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Maps to: CC `services/lsp/types.ts` `LspServerConfig`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspServerConfig {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extension_to_language: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_folder: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_js_integer"
    )]
    pub startup_timeout: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_js_integer"
    )]
    pub max_restarts: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart_on_crash: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_js_integer"
    )]
    pub shutdown_timeout: Option<u64>,
}

/// Typed representation of CC's Number fields in `LspServerConfig`.
/// JSON `1.0`, `1e3` and `-0` remain floating-point Numbers in serde_json,
/// although JS and the canonical schema treat their values as integers.
/// This only projects representable integer values; the source schema still
/// owns positive/nonnegative and safe-integer validation.
fn deserialize_optional_js_integer<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Some(number) = Option::<serde_json::Number>::deserialize(deserializer)? else {
        return Ok(None);
    };
    if let Some(integer) = number.as_u64() {
        return Ok(Some(integer));
    }
    if let Some(number) = number.as_f64()
        && number.is_finite()
        && number.fract() == 0.0
        && number >= 0.0
        && number < u64::MAX as f64
    {
        return Ok(Some(number as u64));
    }
    Err(serde::de::Error::custom(
        "expected an integer representable as u64",
    ))
}

/// Maps to: CC `services/lsp/types.ts` `ScopedLspServerConfig`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedLspServerConfig {
    #[serde(flatten)]
    pub config: LspServerConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Maps to: CC `services/lsp/types.ts` `LspServerState`.
///
/// The rebuild's `types.ts` is a generated stub; the runtime literals are the
/// assignments in `LSPServerInstance.ts` (`stopped|starting|running|stopping|
/// error`, lines 113/154/250/259/280/282/285) plus the documented
/// `stopped → starting → running` machine at `:74-78`. `Idle`/`Initializing`
/// are unassigned on both sides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LspServerState {
    Idle,
    Starting,
    Initializing,
    Running,
    Error,
    Stopping,
    #[default]
    Stopped,
}

impl LspServerState {
    /// The literal CC interpolates into model-facing copy, e.g.
    /// `` `Cannot send request to LSP server '${name}': server is ${state}` ``
    /// (`LSPServerInstance.ts:357-359`). `{:?}` would emit the PascalCase Rust
    /// variant name instead.
    pub fn as_str(self) -> &'static str {
        match self {
            LspServerState::Idle => "idle",
            LspServerState::Starting => "starting",
            LspServerState::Initializing => "initializing",
            LspServerState::Running => "running",
            LspServerState::Error => "error",
            LspServerState::Stopping => "stopping",
            LspServerState::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LspServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Maps to: CC `services/diagnosticTracking.ts` `Diagnostic.range` positions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticPosition {
    pub line: u32,
    pub character: u32,
}

/// Maps to: CC `services/diagnosticTracking.ts` `Diagnostic.range`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRange {
    pub start: DiagnosticPosition,
    pub end: DiagnosticPosition,
}

/// Maps to: CC `services/diagnosticTracking.ts` `Diagnostic`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub severity: String,
    pub range: DiagnosticRange,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Maps to: CC `services/diagnosticTracking.ts` `DiagnosticFile`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticFile {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lsp_integer_carrier_projects_raw_json_number_spellings() {
        for (raw, expected) in [
            ("1.0", 1),
            ("1e3", 1000),
            ("-0", 0),
            ("4294967296.0", 4294967296),
            ("9007199254740991.0", 9007199254740991),
        ] {
            // Test the typed Number adapter separately from text parsing.
            // The end-to-end loader fixture keeps these exact raw spellings;
            // here a correctly rounded f64 exercises the floating carrier.
            let number = serde_json::Number::from_f64(raw.parse::<f64>().unwrap()).unwrap();
            let value = serde_json::json!({
                "command": "x",
                "maxRestarts": number,
                "startupTimeout": number,
                "shutdownTimeout": number,
            });
            let parsed: LspServerConfig = serde_json::from_value(value).unwrap();
            assert_eq!(parsed.max_restarts, Some(expected));
            assert_eq!(parsed.startup_timeout, Some(expected));
            assert_eq!(parsed.shutdown_timeout, Some(expected));
        }
        for raw in ["1.5", "-1", "18446744073709551616.0", "\"1\""] {
            let raw = format!(r#"{{"command":"x","maxRestarts":{raw}}}"#);
            assert!(
                serde_json::from_str::<LspServerConfig>(&raw).is_err(),
                "{raw}"
            );
        }
    }
}
