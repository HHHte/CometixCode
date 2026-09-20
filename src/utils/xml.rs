//! XML escaping helpers.
//!
//! Maps to: CC `utils/xml.ts`.

/// Maps to: CC `utils/xml.ts#escapeXml`.
pub fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Maps to: CC `utils/xml.ts#escapeXmlAttr`.
pub fn escape_xml_attr(value: &str) -> String {
    escape_xml(value)
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_xml_attr_matches_official_replacements() {
        assert_eq!(escape_xml_attr("<&>\"'"), "&lt;&amp;&gt;&quot;&apos;");
    }
}
