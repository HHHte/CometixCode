//! Maps to: CC `components/PromptInput/inputPaste.ts:1-90`.

use regex::Regex;
use std::collections::BTreeMap;

pub const TRUNCATION_THRESHOLD: usize = 10_000;
pub const PREVIEW_LENGTH: usize = 1_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PastedContent {
    Text {
        id: usize,
        content: String,
    },
    Image {
        id: usize,
        media_type: Option<String>,
        /// Base64 bytes are present for live clipboard images. History-only
        /// metadata from older entries may omit them.
        data: Option<String>,
        /// Maps to: CC `utils/config.ts:54-62` `PastedContent.filename`.
        filename: Option<String>,
        /// Maps to: CC `utils/config.ts:54-62` `PastedContent.dimensions`.
        dimensions: Option<crate::utils::image_resizer::ImageDimensions>,
        /// Maps to: CC `utils/config.ts:54-62` `PastedContent.sourcePath`.
        source_path: Option<String>,
    },
}

impl PastedContent {
    pub fn id(&self) -> usize {
        match self {
            Self::Text { id, .. } | Self::Image { id, .. } => *id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TruncatedMessage {
    pub truncated_text: String,
    pub placeholder_content: String,
}

fn utf16_slice(text: &str, start: usize, end: usize) -> String {
    let units = text.encode_utf16().collect::<Vec<_>>();
    String::from_utf16_lossy(&units[start.min(units.len())..end.min(units.len())])
}

pub fn maybe_truncate_message_for_input(text: &str, next_paste_id: usize) -> TruncatedMessage {
    let length = text.encode_utf16().count();
    if length <= TRUNCATION_THRESHOLD {
        return TruncatedMessage {
            truncated_text: text.to_string(),
            placeholder_content: String::new(),
        };
    }
    let start_len = PREVIEW_LENGTH / 2;
    let end_start = length.saturating_sub(PREVIEW_LENGTH / 2);
    let start = utf16_slice(text, 0, start_len);
    let end = utf16_slice(text, end_start, length);
    let placeholder_content = utf16_slice(text, start_len, end_start);
    let lines = placeholder_content.match_indices("\r\n").count()
        + placeholder_content
            .replace("\r\n", "")
            .chars()
            .filter(|ch| matches!(ch, '\r' | '\n'))
            .count();
    let reference = format!("[...Truncated text #{next_paste_id} +{lines} lines...]");
    TruncatedMessage {
        truncated_text: format!("{start}{reference}{end}"),
        placeholder_content,
    }
}

/// Expands collapsed pasted-text chips before the model request while leaving
/// image chips in display text; images become adjacent typed content blocks.
pub fn expand_text_references(input: &str, pasted: &BTreeMap<usize, PastedContent>) -> String {
    let mut expanded = input.to_string();
    for (id, content) in pasted {
        let PastedContent::Text { content, .. } = content else {
            continue;
        };
        let pattern = format!(
            r"\[(?:\.\.\.Truncated text #{id} \+\d+ lines\.\.\.|Pasted text #{id}(?: \+\d+ lines)?)\]"
        );
        if let Ok(reference) = Regex::new(&pattern) {
            expanded = reference
                .replace_all(&expanded, content.as_str())
                .into_owned();
        }
    }
    expanded
}

pub fn prune_unreferenced_images(
    input: &str,
    pasted: &BTreeMap<usize, PastedContent>,
) -> BTreeMap<usize, PastedContent> {
    pasted
        .iter()
        .filter(|(id, content)| {
            !matches!(content, PastedContent::Image { .. })
                || input.contains(&format!("[Image #{id}]"))
        })
        .map(|(id, content)| (*id, content.clone()))
        .collect()
}

pub fn image_content_blocks(
    pasted: &BTreeMap<usize, PastedContent>,
) -> Vec<crate::types::message::UserContent> {
    pasted
        .values()
        .filter_map(|content| match content {
            PastedContent::Image {
                media_type,
                data: Some(data),
                ..
            } if !data.is_empty() => Some(crate::types::message::UserContent::Image {
                media_type: media_type
                    .clone()
                    .unwrap_or_else(|| "image/png".to_string()),
                data: data.clone(),
            }),
            _ => None,
        })
        .collect()
}

pub fn maybe_truncate_input(
    input: &str,
    pasted: &BTreeMap<usize, PastedContent>,
) -> (String, BTreeMap<usize, PastedContent>) {
    let next_id = pasted.keys().next_back().copied().unwrap_or(0) + 1;
    let truncated = maybe_truncate_message_for_input(input, next_id);
    if truncated.placeholder_content.is_empty() {
        return (input.to_string(), pasted.clone());
    }
    let mut next = pasted.clone();
    next.insert(
        next_id,
        PastedContent::Text {
            id: next_id,
            content: truncated.placeholder_content,
        },
    );
    (truncated.truncated_text, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_input_is_identity_and_long_input_keeps_utf16_head_tail() {
        let pasted = BTreeMap::new();
        assert_eq!(maybe_truncate_input("short", &pasted).0, "short");
        let input = format!("{}\n{}", "🙂".repeat(3_000), "z".repeat(5_000));
        let (display, contents) = maybe_truncate_input(&input, &pasted);
        assert!(display.contains("[...Truncated text #1 +1 lines...]"));
        assert_eq!(
            display.encode_utf16().count(),
            1_000 + "[...Truncated text #1 +1 lines...]".encode_utf16().count()
        );
        assert!(matches!(contents.get(&1), Some(PastedContent::Text { .. })));
    }

    #[test]
    fn model_projection_expands_text_chips_and_emits_image_blocks() {
        let pasted = BTreeMap::from([
            (
                1,
                PastedContent::Text {
                    id: 1,
                    content: "full text".to_string(),
                },
            ),
            (
                2,
                PastedContent::Image {
                    id: 2,
                    media_type: Some("image/png".to_string()),
                    data: Some("AAAA".to_string()),
                    filename: Some("Pasted image".to_string()),
                    dimensions: None,
                    source_path: None,
                },
            ),
        ]);
        assert_eq!(
            expand_text_references("before [...Truncated text #1 +9 lines...] after", &pasted),
            "before full text after"
        );
        assert_eq!(image_content_blocks(&pasted).len(), 1);
        assert!(prune_unreferenced_images("[Image #2]", &pasted).contains_key(&2));
        assert!(!prune_unreferenced_images("deleted", &pasted).contains_key(&2));
    }

    #[test]
    fn next_identifier_follows_existing_max() {
        let mut pasted = BTreeMap::new();
        pasted.insert(
            7,
            PastedContent::Image {
                id: 7,
                media_type: None,
                data: None,
                filename: None,
                dimensions: None,
                source_path: None,
            },
        );
        let (_, next) = maybe_truncate_input(&"x".repeat(10_001), &pasted);
        assert!(next.contains_key(&8));
    }
}
