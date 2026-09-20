//! Maps to: CC `components/ClickableImageRef.tsx`.
//!
//! The component resolves stored image paths through the official
//! `utils/imageStore.ts#getStoredImagePath` boundary and emits an OSC-8 file
//! hyperlink only when a path is known and hyperlink support is enabled.

use crate::utils::image_store::{get_stored_image_path, image_path_to_file_url};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ClickableImageRefProps {
    pub image_id: u64,
    pub background_color: Option<Color>,
    pub is_selected: bool,
    /// Maps to CC `supportsHyperlinks()` result. `None` uses iocraft's terminal
    /// capability detection; tests can force either branch deterministically.
    pub supports_hyperlinks: Option<bool>,
}

pub fn clickable_image_ref_display_text(image_id: u64) -> String {
    format!("[Image #{image_id}]")
}

pub fn clickable_image_ref_file_url(image_id: u64, supports_hyperlinks: bool) -> Option<String> {
    if !supports_hyperlinks {
        return None;
    }
    get_stored_image_path(image_id).map(|path| image_path_to_file_url(&path))
}

/// Maps to: CC `components/ClickableImageRef.tsx#ClickableImageRef`.
#[component]
pub fn ClickableImageRef(props: &ClickableImageRefProps) -> impl Into<AnyElement<'static>> {
    let display_text = clickable_image_ref_display_text(props.image_id);
    let supports_hyperlinks = props
        .supports_hyperlinks
        .unwrap_or_else(iocraft::supports_hyperlinks);
    let href = clickable_image_ref_file_url(props.image_id, supports_hyperlinks);
    let is_link = href.is_some();

    element! {
        Text(
            content: display_text,
            background_color: props.background_color,
            inverse: props.is_selected,
            bold: props.is_selected && is_link,
            href: href,
            wrap: TextWrap::NoWrap,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::image_store::{cache_stored_image_path, clear_stored_image_paths};
    use std::path::PathBuf;

    #[test]
    fn clickable_image_ref_display_text_matches_official_copy() {
        assert_eq!(clickable_image_ref_display_text(12), "[Image #12]");
    }

    #[test]
    fn clickable_image_ref_returns_file_url_only_when_supported_and_cached() {
        clear_stored_image_paths();
        assert_eq!(clickable_image_ref_file_url(1, true), None);
        cache_stored_image_path(1, PathBuf::from("/tmp/Image #1.png"));
        assert_eq!(
            clickable_image_ref_file_url(1, true),
            Some("file:///tmp/Image%20%231.png".to_string())
        );
        assert_eq!(clickable_image_ref_file_url(1, false), None);
        clear_stored_image_paths();
    }

    #[test]
    fn clickable_image_ref_renders_fallback_without_hyperlink() {
        clear_stored_image_paths();
        let canvas = element! {
            ClickableImageRef(image_id: 2u64, supports_hyperlinks: Some(false))
        }
        .render(Some(40));

        assert_eq!(canvas.to_string().trim_end(), "[Image #2]");
        assert_eq!(canvas.hyperlink_at(1, 0), None);
    }

    #[test]
    fn clickable_image_ref_renders_file_hyperlink_when_cached() {
        clear_stored_image_paths();
        cache_stored_image_path(3, PathBuf::from("/tmp/three.png"));
        let canvas = element! {
            ClickableImageRef(image_id: 3u64, supports_hyperlinks: Some(true), is_selected: true)
        }
        .render(Some(40));

        assert_eq!(canvas.to_string().trim_end(), "[Image #3]");
        assert_eq!(
            canvas.hyperlink_at(1, 0).as_deref(),
            Some("file:///tmp/three.png")
        );
        assert_eq!(
            canvas.resolved_text_style(0, 0).expect("style").weight,
            Weight::Bold,
            "official link branch bolds selected image chips"
        );
        clear_stored_image_paths();
    }
}
