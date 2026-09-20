//! Session-scoped cache of rendered Tool base schemas.
//!
//! Maps to: CC `utils/toolSchemaCache.ts`.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};

/// Maps to: CC `utils/toolSchemaCache.ts:15-18` `CachedSchema`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CachedSchema {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) input_schema: serde_json::Value,
    pub(crate) strict: Option<bool>,
    pub(crate) eager_input_streaming: Option<bool>,
}

static TOOL_SCHEMA_CACHE: LazyLock<Mutex<HashMap<String, CachedSchema>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Maps to: CC `utils/toolSchemaCache.ts:20-22` `getToolSchemaCache`.
pub(crate) fn get_tool_schema_cache() -> MutexGuard<'static, HashMap<String, CachedSchema>> {
    TOOL_SCHEMA_CACHE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Maps to: CC `utils/toolSchemaCache.ts:24-26` `clearToolSchemaCache`.
pub fn clear_tool_schema_cache() {
    get_tool_schema_cache().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_tool_schema_cache_matches_official_process_cache_lifecycle() {
        clear_tool_schema_cache();
        get_tool_schema_cache().insert(
            "Read".to_string(),
            CachedSchema {
                name: "Read".to_string(),
                description: "Read a file".to_string(),
                input_schema: serde_json::json!({"type":"object"}),
                strict: Some(true),
                eager_input_streaming: None,
            },
        );
        assert_eq!(get_tool_schema_cache().len(), 1);
        clear_tool_schema_cache();
        assert!(get_tool_schema_cache().is_empty());
    }
}
