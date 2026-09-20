//! The settings JSON Schema handed to the model on failed settings edits.
//!
//! Maps to: CC `utils/settings/schemaOutput.ts` — `toJSONSchema(SettingsSchema())`
//! pretty-printed. Computed from the live carrier schema, so the output tracks
//! the same build-time gates (XAA env, USER_TYPE, feature flags) the schema
//! itself froze at first access.

use super::types::settings_schema;
use crate::utils::zod::to_json_schema;

/// Maps to: CC `generateSettingsJSONSchema()`.
pub fn generate_settings_json_schema() -> String {
    let json_schema = to_json_schema(settings_schema());
    serde_json::to_string_pretty(&json_schema).unwrap_or_default()
}
