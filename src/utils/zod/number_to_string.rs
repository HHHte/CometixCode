//! ECMAScript `Number::toString` for the carrier.
//!
//! Maps to: the `String(n)` JS performs when a parsed number is written back
//! out — the path CC's tools hit whenever a numeric value is re-emitted into a
//! transcript, an API payload, or an error string. Delegates the digit work to
//! `ryu-js` (the ECMAScript-correct shortest-round-trip formatter) and adds the
//! two rules ryu-js does not own: `-0 → "0"` (JS has no negative-zero string)
//! and the `1e21` threshold where JS switches to exponent form.
//!
//! Single funnel: the crate already calls `ryu_js::Buffer::format` in several
//! places (`read_file_in_range.rs`, `file.rs`, `mcp_validation.rs`) — each
//! open-coding the same value-to-string match. This is the one place that
//! behaviour should live.

/// Maps to: ECMAScript `Number::toString()` for a finite f64.
///
/// `ryu_js::Buffer::format` already produces the shortest round-trip digits
/// and the exponent form past the 1e21 threshold; what it does not do is the
/// sign rule — JS stringifies `-0` as `"0"`.
pub fn javascript_number_to_string(value: f64) -> String {
    if value == 0.0 {
        // Covers both 0.0 and -0.0 (`0.0 == -0.0` in IEEE), matching JS.
        return "0".to_string();
    }
    ryu_js::Buffer::new().format(value).to_string()
}

#[cfg(test)]
mod tests {
    use super::javascript_number_to_string;

    /// Golden values sampled from real `String(n)` in a JS engine (bun).
    #[test]
    fn matches_ecmascript_number_to_string() {
        let cases: &[(f64, &str)] = &[
            (0.0, "0"),
            (-0.0, "0"), // JS has no "-0" string
            (1.0, "1"),
            (1.5, "1.5"),
            (30.0, "30"),
            (1e21, "1e+21"),
            (1e20, "100000000000000000000"), // 21-digit threshold, still decimal
            (1e-7, "1e-7"),
            (0.000001, "0.000001"),
            (123456789.0, "123456789"),
            (1.7976931348623157e308, "1.7976931348623157e+308"),
            (5e-324, "5e-324"),
            (0.1 + 0.2, "0.30000000000000004"),
            (-42.5, "-42.5"),
            (9007199254740991.0, "9007199254740991"),
        ];
        for (input, want) in cases {
            assert_eq!(&javascript_number_to_string(*input), want, "input {input}");
        }
    }
}
