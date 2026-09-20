//! Maps to: CC `components/FilePathLink.tsx`.
//!
//! Renders an absolute file path as a `file://` OSC 8 hyperlink, preserving the
//! official component boundary that helps terminals recognize paths inside
//! punctuation.

use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct FilePathLinkProps {
    pub file_path: String,
    /// Optional display text; defaults to `file_path` like official `children`.
    pub label: Option<String>,
    /// Test/control seam for terminals without hyperlink support.
    pub enabled: Option<bool>,
}

/// Maps to: CC `pathToFileURL(filePath).href` used by `FilePathLink`.
pub fn file_path_to_file_url(file_path: &str) -> String {
    if file_path.starts_with("file://") {
        return file_path.to_string();
    }

    #[cfg(windows)]
    {
        let normalized = file_path.replace('\\', "/");
        if normalized.starts_with("//") {
            return format!("file:{}", percent_encode_file_url_path(&normalized));
        }
        return format!("file:///{}", percent_encode_file_url_path(&normalized));
    }

    #[cfg(not(windows))]
    {
        format!("file://{}", percent_encode_file_url_path(file_path))
    }
}

fn percent_encode_file_url_path(path: &str) -> String {
    let mut out = String::new();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Internal representation adapter for the `Explicit structured Ink text-flow carrier` L1
/// mapping of CC `components/FilePathLink.tsx#FilePathLink:17-19`.
///
/// FilePathLink retains path-to-URL ownership and delegates Link's terminal
/// support/fallback branch to iocraft's source-shaped Link carrier.
pub(crate) fn file_path_link_segment(
    file_path: &str,
    label: Option<String>,
    enabled: Option<bool>,
) -> StyledSegment {
    let label = label.unwrap_or_else(|| file_path.to_string());
    link_segment(file_path_to_file_url(file_path), Some(label), None, enabled)
}

/// Maps to: CC `components/FilePathLink.tsx#FilePathLink`.
#[component]
pub fn FilePathLink(props: &FilePathLinkProps) -> impl Into<AnyElement<'static>> {
    let label = props
        .label
        .clone()
        .unwrap_or_else(|| props.file_path.clone());
    element! {
        Link(
            url: file_path_to_file_url(&props.file_path),
            label: Some(label),
            enabled: props.enabled,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_path_link_builds_file_url_like_path_to_file_url() {
        assert_eq!(
            file_path_to_file_url("/tmp/hello world#1.rs"),
            "file:///tmp/hello%20world%231.rs"
        );
    }

    #[test]
    fn file_path_link_renders_label_and_hyperlink_metadata() {
        let canvas = element! {
            FilePathLink(
                file_path: "/tmp/demo.rs".to_string(),
                label: Some("demo.rs".to_string()),
                enabled: Some(true),
            )
        }
        .render(Some(80));

        assert_eq!(canvas.to_string().trim_end(), "demo.rs");
        assert_eq!(
            canvas.hyperlink_at(0, 0).as_deref(),
            Some("file:///tmp/demo.rs")
        );
    }

    /// Maps to CC `FilePathLink.tsx:17-19` plus `ink/components/Link.tsx:20-30`.
    #[test]
    fn file_path_link_segment_matches_standalone_link_support_and_fallback() {
        let enabled =
            file_path_link_segment("/tmp/demo.rs", Some("demo.rs".to_string()), Some(true));
        assert_eq!(enabled.text, "demo.rs");
        assert_eq!(enabled.hyperlink.as_deref(), Some("file:///tmp/demo.rs"));

        let disabled =
            file_path_link_segment("/tmp/demo.rs", Some("demo.rs".to_string()), Some(false));
        assert_eq!(disabled.text, "demo.rs");
        assert_eq!(disabled.hyperlink, None);

        let standalone = element! {
            FilePathLink(
                file_path: "/tmp/demo.rs".to_string(),
                label: Some("demo.rs".to_string()),
                enabled: Some(false),
            )
        }
        .render(Some(80));
        assert_eq!(standalone.to_string().trim_end(), disabled.text);
        assert_eq!(standalone.hyperlink_at(0, 0), None);
    }
}
