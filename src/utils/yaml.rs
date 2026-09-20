//! Maps to: CC `utils/yaml.ts`.
//!
//! Native parser adapter: the source delegates to Bun.YAML / npm yaml; Rust
//! uses the existing serde_yaml dependency. Parser diagnostics retain native
//! wording. Do not silently recover, retry, or replace errors at this boundary.

/// Maps to: CC `utils/yaml.ts:9-15#parseYaml`.
/// The YAML value is kept unprojected so callers can distinguish scalar,
/// sequence, mapping and null (including nonfinite numbers).
pub fn parse_yaml(input: &str) -> Result<serde_yaml::Value, serde_yaml::Error> {
    serde_yaml::from_str::<YamlValue>(input).map(|value| value.0)
}

// Native deserializer representation adapter: serde_yaml::Value's default
// mapping visitor rejects duplicate entries, while Bun.YAML keeps the last
// value. Reuse the parser's tokenization/resolution and preserve every other
// error; do not preprocess YAML text or retry rejected documents.
struct YamlValue(serde_yaml::Value);
impl<'de> serde::Deserialize<'de> for YamlValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
        struct ValueVisitor;
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = YamlValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a YAML value")
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::Bool(value)))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::Number(value.into())))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::Number(value.into())))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::Number(value.into())))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::String(value.into())))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::String(value)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<YamlValue, E> {
                Ok(YamlValue(serde_yaml::Value::Null))
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<YamlValue, E> {
                self.visit_unit()
            }
            fn visit_some<D: serde::Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<YamlValue, D::Error> {
                serde::Deserialize::deserialize(deserializer)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut data: A) -> Result<YamlValue, A::Error> {
                let mut sequence = Vec::new();
                while let Some(YamlValue(value)) = data.next_element()? {
                    sequence.push(value);
                }
                Ok(YamlValue(serde_yaml::Value::Sequence(sequence)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut data: A) -> Result<YamlValue, A::Error> {
                let mut mapping = serde_yaml::Mapping::new();
                while let Some((YamlValue(key), YamlValue(value))) = data.next_entry()? {
                    mapping.insert(key, value);
                }
                Ok(YamlValue(serde_yaml::Value::Mapping(mapping)))
            }
            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<YamlValue, A::Error> {
                let (_, content) = data.variant::<String>()?;
                // Bun's YAML-to-JS value has no tagged-value wrapper.
                content.newtype_variant()
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yaml_duplicate_keys_nested_values_and_aliases_match_bun() {
        let value=parse_yaml("name: first\nname: last\nmap: {a: 1, a: 2}\nlist: [{b: 3, b: 4}]\nanchor: &v {c: 5, c: 6}\ncopy: *v").unwrap();
        assert_eq!(value["name"].as_str(), Some("last"));
        assert_eq!(value["map"]["a"].as_i64(), Some(2));
        assert_eq!(value["list"][0]["b"].as_i64(), Some(4));
        assert_eq!(value["copy"]["c"].as_i64(), Some(6));
        assert_eq!(value["anchor"], value["copy"]);
        assert_eq!(
            parse_yaml("description: !custom value").unwrap()["description"].as_str(),
            Some("value")
        );
    }

    #[test]
    fn parse_yaml_keeps_value_kinds_and_errors_for_validation() {
        assert!(parse_yaml("").unwrap().is_null());
        assert!(parse_yaml("- one").unwrap().is_sequence());
        assert!(parse_yaml("true").unwrap().is_bool());
        assert!(parse_yaml("description: null").unwrap().is_mapping());
        // Bun reports `YAML Parse error: Unexpected token`; serde_yaml gives
        // its own location-bearing diagnostic. Both errors propagate intact.
        let error = parse_yaml("description: [").unwrap_err();
        assert!(!error.to_string().is_empty());
    }
}
