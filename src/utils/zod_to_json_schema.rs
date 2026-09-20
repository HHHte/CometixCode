//! Maps to: CC `utils/zodToJsonSchema.ts` — the per-reference caching wrapper
//! around the JSON Schema projection.
//!
//! CC's file is a thin wrapper: it caches by schema identity and delegates the
//! actual conversion to zod/v4's native `toJSONSchema`. It owns NO projection
//! logic itself. The Rust split mirrors that exactly:
//!   - the projection (`zod/v4 toJSONSchema`) lives in the carrier at
//!     `utils/zod::to_json_schema`;
//!   - this file owns only CC's caching wrapper (`utils/zodToJsonSchema.ts`).
//!
//! CC caches in a `WeakMap` keyed by the schema object, because
//! `toolToAPISchema()` runs it for every tool on every API request (~60-250
//! times/turn) and `lazySchema()` guarantees one schema reference per session.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// CC `JsonSchema7Type`.
pub type JsonSchema7Type = serde_json::Value;

fn cache() -> &'static Mutex<HashMap<usize, JsonSchema7Type>> {
    static CACHE: OnceLock<Mutex<HashMap<usize, JsonSchema7Type>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Maps to: CC `zodToJsonSchema(schema)`.
///
/// Rust has no object-identity WeakMap; the cache key is the schema's address,
/// which is stable because every schema lives in a per-tool
/// `static OnceLock<Schema>` — the Rust idiom standing in for CC's
/// `lazySchema()` memoized factory (first-access build, process-wide single
/// instance; the language primitive covers what CC needed a helper for).
/// Callers must pass the shared `&'static` reference, not a re-built clone
/// (a fresh address would miss the cache and, worse, could collide with a
/// freed one).
pub fn zod_to_json_schema(schema: &'static crate::utils::zod::Schema) -> JsonSchema7Type {
    let key = schema as *const _ as usize;
    if let Some(hit) = cache().lock().unwrap().get(&key) {
        return hit.clone();
    }
    let result = crate::utils::zod::to_json_schema(schema);
    cache().lock().unwrap().insert(key, result.clone());
    result
}
