//! Tolerant number preprocessing for model-generated tool inputs.
//!
//! Maps to: CC `utils/semanticNumber.ts:1-34`.
//! The API schema remains `number`. Before validation, only finite decimal
//! string literals matching `^-?\d+(\.\d+)?$` are converted; whitespace,
//! signs such as `+`, exponents, empty strings, and non-finite values remain
//! unchanged so normal schema validation rejects them.

fn is_decimal_literal(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut index = usize::from(bytes[0] == b'-');
    if index == bytes.len() {
        return false;
    }
    let integer_start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index == integer_start {
        return false;
    }
    if index == bytes.len() {
        return true;
    }
    if bytes[index] != b'.' {
        return false;
    }
    index += 1;
    let fraction_start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    index == bytes.len() && index > fraction_start
}

fn json_number(value: &str) -> Option<serde_json::Number> {
    let parsed = value.parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }

    // Preserve integer representation for the common tool-input case so
    // `as_u64` / `as_i64` consumers see the same JS Number value. Parsing via
    // f64 first intentionally applies JavaScript Number-style rounding.
    if parsed.fract() == 0.0 {
        if parsed >= i64::MIN as f64 && parsed < i64::MAX as f64 {
            return Some(serde_json::Number::from(parsed as i64));
        }
        if parsed >= 0.0 && parsed < u64::MAX as f64 {
            return Some(serde_json::Number::from(parsed as u64));
        }
    }
    serde_json::Number::from_f64(parsed)
}

/// Apply CC `semanticNumber` preprocessing to one JSON value in place.
pub fn preprocess(value: &mut serde_json::Value) {
    let Some(literal) = value.as_str() else {
        return;
    };
    if !is_decimal_literal(literal) {
        return;
    }
    if let Some(number) = json_number(literal) {
        *value = serde_json::Value::Number(number);
    }
}

/// Maps to: CC `utils/semanticNumber.ts:26-33` `semanticNumber(inner)`.
///
/// Builds a carrier `preprocess` node (`z.preprocess(f, inner)`), defaulting
/// the inner schema to `z.number()` exactly as CC defaults to `z.number()`:
/// the decimal tolerance runs before validation, and `to_json_schema` passes
/// through to `inner`, so the model still sees `{"type":"number"}` — the
/// string tolerance is invisible client-side coercion, not an advertised input
/// shape. `.optional()` / `.default()` go INSIDE (on the inner schema), as in
/// CC.
pub fn semantic_number(inner: crate::utils::zod::Schema) -> crate::utils::zod::Schema {
    crate::utils::zod::preprocess(
        |value| {
            let mut value = value.clone();
            preprocess(&mut value);
            value
        },
        inner,
    )
}

/// CC's default-argument form `semanticNumber()` ≙ `semanticNumber(z.number())`.
pub fn semantic_number_default() -> crate::utils::zod::Schema {
    semantic_number(crate::utils::zod::number())
}

/// Apply [`preprocess`] to an object property when it is present.
pub fn preprocess_object_field(input: &mut serde_json::Value, field: &str) {
    if let Some(value) = input.get_mut(field) {
        preprocess(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_number_matches_official_decimal_literal_tolerance() {
        for (input, expected) in [
            (serde_json::json!("30"), serde_json::json!(30)),
            (serde_json::json!("-5"), serde_json::json!(-5)),
            (serde_json::json!("3.25"), serde_json::json!(3.25)),
            (serde_json::json!("30.0"), serde_json::json!(30)),
            (serde_json::json!("00"), serde_json::json!(0)),
            (serde_json::json!(7), serde_json::json!(7)),
        ] {
            let mut actual = input;
            preprocess(&mut actual);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn semantic_number_leaves_non_official_literals_for_schema_rejection() {
        for input in [
            serde_json::json!(""),
            serde_json::json!(" 30"),
            serde_json::json!("+5"),
            serde_json::json!(".5"),
            serde_json::json!("5."),
            serde_json::json!("1e3"),
            serde_json::json!("NaN"),
            serde_json::Value::Null,
            serde_json::json!(true),
        ] {
            let mut actual = input.clone();
            preprocess(&mut actual);
            assert_eq!(actual, input);
        }
    }

    /// The schema-facing constructor is the file's subject (CC
    /// `semanticNumber(inner)`): it builds a carrier preprocess node over the
    /// inner schema. Runs through the carrier's own `safe_parse`.
    #[test]
    fn semantic_number_node_coerces_decimal_strings_through_the_carrier() {
        use crate::utils::zod::{safe_parse, to_json_schema};
        let schema = semantic_number(crate::utils::zod::number().int().optional());
        // quoted decimal → coerced before validation
        let parsed = safe_parse(&schema, &serde_json::json!("30")).unwrap();
        assert_eq!(parsed, serde_json::json!(30));
        // non-literal passes through and is rejected by the inner schema
        assert!(safe_parse(&schema, &serde_json::json!("1e3")).is_err());
        // the projection passes through to the inner schema: the model still
        // sees a number, the string tolerance invisible (CC doc).
        assert_eq!(
            to_json_schema(&schema)["type"],
            serde_json::json!("integer")
        );
    }

    /// CC's default-argument form `semanticNumber()` ≙ `semanticNumber(z.number())`.
    #[test]
    fn semantic_number_default_wraps_a_plain_number() {
        use crate::utils::zod::safe_parse;
        let schema = semantic_number_default();
        assert_eq!(
            safe_parse(&schema, &serde_json::json!("-5")).unwrap(),
            serde_json::json!(-5)
        );
        assert_eq!(
            safe_parse(&schema, &serde_json::json!(7)).unwrap(),
            serde_json::json!(7)
        );
    }
}
