//! Tolerant boolean preprocessing for model-generated tool inputs.
//!
//! Maps to: CC `utils/semanticBoolean.ts:1-28`.
//! The API schema remains `boolean`; only the exact string literals `"true"`
//! and `"false"` are converted before schema validation. This deliberately
//! does not use truthiness and does not accept case, whitespace, `yes`, or `1`.

/// Apply CC `semanticBoolean` preprocessing to one JSON value in place.
pub fn preprocess(value: &mut serde_json::Value) {
    let replacement = match value.as_str() {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    };
    if let Some(replacement) = replacement {
        *value = serde_json::Value::Bool(replacement);
    }
}

/// Apply [`preprocess`] to an object property when it is present.
pub fn preprocess_object_field(input: &mut serde_json::Value, field: &str) {
    if let Some(value) = input.get_mut(field) {
        preprocess(value);
    }
}

/// Maps to: CC `utils/semanticBoolean.ts:26-28` `semanticBoolean(inner)`.
///
/// Builds a carrier `preprocess` node (`z.preprocess(f, inner)`), defaulting
/// the inner schema to `z.boolean()` exactly as CC defaults to `z.boolean()`:
/// the true/false-literal tolerance runs before validation, and
/// `to_json_schema` passes through to `inner`, so the model still sees
/// `{"type":"boolean"}`. `.optional()` / `.default()` go INSIDE (on the inner
/// schema), as in CC.
pub fn semantic_boolean(inner: crate::utils::zod::Schema) -> crate::utils::zod::Schema {
    crate::utils::zod::preprocess(
        |value| {
            let mut value = value.clone();
            preprocess(&mut value);
            value
        },
        inner,
    )
}

/// CC's default-argument form `semanticBoolean()` ≙ `semanticBoolean(z.boolean())`.
pub fn semantic_boolean_default() -> crate::utils::zod::Schema {
    semantic_boolean(crate::utils::zod::boolean())
}

/// Read a native or semantically tolerated boolean without broadening the
/// official accepted literals. Direct tool helpers use this only at boundaries
/// that may be called outside the canonical schema parser in tests.
pub fn parse_json_bool(value: &serde_json::Value) -> Option<bool> {
    match value {
        serde_json::Value::Bool(value) => Some(*value),
        serde_json::Value::String(value) if value == "true" => Some(true),
        serde_json::Value::String(value) if value == "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_boolean_matches_official_exact_string_tolerance() {
        for (input, expected) in [
            (serde_json::json!(true), serde_json::json!(true)),
            (serde_json::json!(false), serde_json::json!(false)),
            (serde_json::json!("true"), serde_json::json!(true)),
            (serde_json::json!("false"), serde_json::json!(false)),
        ] {
            let mut actual = input;
            preprocess(&mut actual);
            assert_eq!(actual, expected);
        }

        for input in [
            serde_json::json!("TRUE"),
            serde_json::json!(" false "),
            serde_json::json!("yes"),
            serde_json::json!("1"),
            serde_json::json!(1),
            serde_json::Value::Null,
        ] {
            let mut actual = input.clone();
            preprocess(&mut actual);
            assert_eq!(actual, input);
            assert_eq!(parse_json_bool(&actual), None);
        }
    }
}
