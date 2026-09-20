//! Maps to: CC `components/PromptInput/ShimmeredInput.tsx:1-122`.
//!
//! Preserves ANSI-visible highlight resolution, per-line layout and the scoped
//! shimmer sweep. Highlight offsets use JavaScript UTF-16 units and exclude
//! ANSI control sequences, matching CC's `HighlightSegmenter`.

use crate::utils::theme::{Theme, ThemeColorKey};
use iocraft::prelude::*;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextHighlight {
    pub start: usize,
    pub end: usize,
    pub color: Option<ThemeColorKey>,
    pub dim_color: bool,
    pub inverse: bool,
    pub shimmer_color: Option<ThemeColorKey>,
    pub priority: i32,
}

#[derive(Clone, Debug)]
struct Segment {
    text: String,
    start: usize,
    highlight: Option<TextHighlight>,
}

#[derive(Clone, Debug)]
enum AnsiToken {
    Code(String),
    Text(Vec<u16>),
}

fn ansi_tokens(text: &str) -> Vec<AnsiToken> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut text_start = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != 0x1b {
            index += 1;
            continue;
        }
        if text_start < index {
            tokens.push(AnsiToken::Text(
                text[text_start..index].encode_utf16().collect(),
            ));
        }
        let start = index;
        index += 1;
        if index < bytes.len() && bytes[index] == b'[' {
            index += 1;
            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;
                if (0x40..=0x7e).contains(&byte) {
                    break;
                }
            }
        } else if index < bytes.len() && bytes[index] == b']' {
            index += 1;
            while index < bytes.len() {
                if bytes[index] == 0x07 {
                    index += 1;
                    break;
                }
                if bytes[index] == 0x1b && index + 1 < bytes.len() && bytes[index + 1] == b'\\' {
                    index += 2;
                    break;
                }
                index += 1;
            }
        } else if index < bytes.len() {
            index += 1;
        }
        tokens.push(AnsiToken::Code(text[start..index].to_string()));
        text_start = index;
    }
    if text_start < text.len() {
        tokens.push(AnsiToken::Text(text[text_start..].encode_utf16().collect()));
    }
    tokens
}

#[derive(Default)]
struct ActiveAnsiCodes {
    sgr: Vec<String>,
    hyperlink: Option<String>,
}

impl ActiveAnsiCodes {
    fn apply(&mut self, code: &str) {
        if code.starts_with("\x1b[") && code.ends_with('m') {
            let params = &code[2..code.len() - 1];
            let values = params
                .split(';')
                .filter_map(|value| value.parse::<u16>().ok())
                .collect::<Vec<_>>();
            let mut index = 0usize;
            let mut resets = params.is_empty();
            while index < values.len() {
                if values[index] == 0 {
                    resets = true;
                    break;
                }
                if matches!(values[index], 38 | 48 | 58) {
                    index += match values.get(index + 1) {
                        Some(2) => 5,
                        Some(5) => 3,
                        _ => 1,
                    };
                } else {
                    index += 1;
                }
            }
            if resets {
                self.sgr.clear();
            } else {
                // Keeping the reduced sequence list is visually equivalent to
                // ansi-tokenize's reduced code state. End codes remove the
                // corresponding category; replacement colors supersede older
                // values when the segment is parsed by `Ansi`.
                self.sgr.push(code.to_string());
            }
        } else if code.starts_with("\x1b]8;;") {
            let target = code
                .strip_prefix("\x1b]8;;")
                .unwrap_or_default()
                .trim_end_matches('\u{7}')
                .trim_end_matches("\x1b\\");
            if target.is_empty() {
                self.hyperlink = None;
            } else {
                self.hyperlink = Some(code.to_string());
            }
        }
    }

    fn prefix(&self) -> String {
        let mut out = self.sgr.concat();
        if let Some(link) = &self.hyperlink {
            out.push_str(link);
        }
        out
    }

    fn suffix(&self) -> String {
        let mut out = String::new();
        if self.hyperlink.is_some() {
            out.push_str("\x1b]8;;\x1b\\");
        }
        if !self.sgr.is_empty() {
            out.push_str("\x1b[0m");
        }
        out
    }
}

struct AnsiHighlightSegmenter {
    tokens: Vec<AnsiToken>,
    token_index: usize,
    char_index: usize,
    visible_position: usize,
    codes: ActiveAnsiCodes,
}

impl AnsiHighlightSegmenter {
    fn new(text: &str) -> Self {
        Self {
            tokens: ansi_tokens(text),
            token_index: 0,
            char_index: 0,
            visible_position: 0,
            codes: ActiveAnsiCodes::default(),
        }
    }

    fn segment_to(&mut self, target: usize) -> Option<Segment> {
        if self.token_index >= self.tokens.len() || target <= self.visible_position {
            return None;
        }
        while let Some(AnsiToken::Code(code)) = self.tokens.get(self.token_index) {
            self.codes.apply(code);
            self.token_index += 1;
        }
        if self.token_index >= self.tokens.len() {
            return None;
        }
        let visible_start = self.visible_position;
        let prefix = self.codes.prefix();
        let mut body = String::new();
        while self.visible_position < target && self.token_index < self.tokens.len() {
            match &self.tokens[self.token_index] {
                AnsiToken::Code(code) => {
                    body.push_str(code);
                    self.codes.apply(code);
                    self.token_index += 1;
                }
                AnsiToken::Text(units) => {
                    let available = units.len().saturating_sub(self.char_index);
                    let wanted = target.saturating_sub(self.visible_position);
                    let take = available.min(wanted);
                    if take == 0 {
                        break;
                    }
                    body.push_str(&String::from_utf16_lossy(
                        &units[self.char_index..self.char_index + take],
                    ));
                    self.char_index += take;
                    self.visible_position += take;
                    if self.char_index == units.len() {
                        self.char_index = 0;
                        self.token_index += 1;
                    }
                }
            }
        }
        if body.is_empty() {
            return None;
        }
        Some(Segment {
            text: format!("{prefix}{body}{}", self.codes.suffix()),
            start: visible_start,
            highlight: None,
        })
    }
}

fn visible_utf16_len(text: &str) -> usize {
    ansi_tokens(text)
        .into_iter()
        .map(|token| match token {
            AnsiToken::Code(_) => 0,
            AnsiToken::Text(units) => units.len(),
        })
        .sum()
}

fn segment_text_by_highlights(text: &str, highlights: &[TextHighlight]) -> Vec<Segment> {
    if highlights.is_empty() {
        return vec![Segment {
            text: text.to_string(),
            start: 0,
            highlight: None,
        }];
    }
    let mut sorted = highlights.to_vec();
    sorted.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.priority.cmp(&a.priority))
    });
    let mut accepted = Vec::<TextHighlight>::new();
    for highlight in sorted {
        if highlight.start == highlight.end
            || accepted
                .iter()
                .any(|used| highlight.start < used.end && highlight.end > used.start)
        {
            continue;
        }
        accepted.push(highlight);
    }
    accepted.sort_by_key(|highlight| highlight.start);

    let mut segmenter = AnsiHighlightSegmenter::new(text);
    let mut out = Vec::new();
    for highlight in accepted {
        if let Some(before) = segmenter.segment_to(highlight.start) {
            out.push(before);
        }
        if let Some(mut highlighted) = segmenter.segment_to(highlight.end) {
            highlighted.highlight = Some(highlight);
            out.push(highlighted);
        }
    }
    if let Some(after) = segmenter.segment_to(usize::MAX) {
        out.push(after);
    }
    if out.is_empty() {
        out.push(Segment {
            text: text.to_string(),
            start: 0,
            highlight: None,
        });
    }
    out
}

#[derive(Default, Props)]
pub struct HighlightedInputProps {
    pub text: String,
    pub highlights: Vec<TextHighlight>,
}

#[component]
pub fn HighlightedInput(
    props: &HighlightedInputProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let segments = segment_text_by_highlights(&props.text, &props.highlights);
    let shimmer = props
        .highlights
        .iter()
        .filter(|highlight| highlight.shimmer_color.is_some())
        .collect::<Vec<_>>();
    let has_shimmer = !shimmer.is_empty();
    let (sweep_start, cycle_length) = if has_shimmer {
        let lo = shimmer
            .iter()
            .map(|highlight| highlight.start)
            .min()
            .unwrap_or(0);
        let hi = shimmer
            .iter()
            .map(|highlight| highlight.end)
            .max()
            .unwrap_or(lo);
        (lo.saturating_sub(10), (hi - lo + 20).max(1))
    } else {
        (0, 1)
    };
    let frame = hooks.use_animation_frame(has_shimmer.then_some(Duration::from_millis(50)));
    let glimmer = if has_shimmer {
        sweep_start + ((frame.time_ms / 50) as usize % cycle_length)
    } else {
        usize::MAX
    };
    let mut lines: Vec<Vec<(String, usize, Option<TextHighlight>)>> = vec![Vec::new()];
    for segment in segments {
        let mut offset = 0usize;
        for (index, part) in segment.text.split('\n').enumerate() {
            if index > 0 {
                lines.push(Vec::new());
                offset += 1;
            }
            if !part.is_empty() {
                lines.last_mut().unwrap().push((
                    part.to_string(),
                    segment.start + offset,
                    segment.highlight.clone(),
                ));
            }
            offset += visible_utf16_len(part);
        }
    }
    let rows = lines.into_iter().map(|parts| {
        if parts.is_empty() { return element! { View { Text(content: " ".to_string()) } }; }
        let items = parts.into_iter().flat_map(|(text, start, highlight)| {
            if let Some(highlight) = highlight {
                if let (Some(message), Some(shimmer)) = (highlight.color, highlight.shimmer_color) {
                    let visible = ansi_tokens(&text)
                        .into_iter()
                        .filter_map(|token| match token {
                            AnsiToken::Code(_) => None,
                            AnsiToken::Text(units) => Some(String::from_utf16_lossy(&units)),
                        })
                        .collect::<String>();
                    return visible.chars().enumerate().map(|(index, ch)| {
                        let absolute = start + visible[..visible.char_indices().nth(index).map(|(at, _)| at).unwrap_or(visible.len())].encode_utf16().count();
                        let key = if absolute.abs_diff(glimmer) <= 1 { shimmer } else { message };
                        element! { Text(content: ch.to_string(), color: theme.color(key)) }.into_any()
                    }).collect::<Vec<_>>();
                }
                return vec![element! {
                    Ansi(
                        content: text,
                        color: highlight.color.map(|key| theme.color(key)),
                        dim_color: highlight.dim_color,
                        invert: highlight.inverse,
                        wrap: TextWrap::NoWrap,
                    )
                }.into_any()];
            }
            vec![element! { Ansi(content: text) }.into_any()]
        }).collect::<Vec<_>>();
        element! { View(flex_direction: FlexDirection::Row) { #(items) } }
    }).collect::<Vec<_>>();
    element! { View(flex_direction: FlexDirection::Column) { #(rows) } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiline_and_overlap_resolution_preserve_high_priority_first_range() {
        let highlights = vec![
            TextHighlight {
                start: 0,
                end: 4,
                color: Some(ThemeColorKey::Warning),
                dim_color: false,
                inverse: false,
                shimmer_color: None,
                priority: 2,
            },
            TextHighlight {
                start: 1,
                end: 6,
                color: Some(ThemeColorKey::Error),
                dim_color: false,
                inverse: false,
                shimmer_color: None,
                priority: 1,
            },
        ];
        let text = element! { ContextProvider(value: Context::owned(*crate::utils::theme::current())) { HighlightedInput(text: "test\nnext".to_string(), highlights: highlights) } }.render(Some(40)).to_string();
        assert!(text.contains("test"));
        assert!(text.contains("next"));
        assert_eq!(text.lines().count(), 2);
    }

    #[test]
    fn ansi_codes_do_not_consume_visible_highlight_offsets() {
        let highlight = TextHighlight {
            start: 1,
            end: 3,
            color: Some(ThemeColorKey::Warning),
            dim_color: false,
            inverse: false,
            shimmer_color: None,
            priority: 1,
        };
        let segments = segment_text_by_highlights("\x1b[31mab\x1b[0mcd", &[highlight]);
        let visible = |text: &str| {
            ansi_tokens(text)
                .into_iter()
                .filter_map(|token| match token {
                    AnsiToken::Code(_) => None,
                    AnsiToken::Text(units) => Some(String::from_utf16_lossy(&units)),
                })
                .collect::<String>()
        };
        assert_eq!(
            segments
                .iter()
                .map(|segment| segment.start)
                .collect::<Vec<_>>(),
            vec![0, 1, 3]
        );
        assert_eq!(visible(&segments[0].text), "a");
        assert_eq!(visible(&segments[1].text), "bc");
        assert_eq!(visible(&segments[2].text), "d");
        assert!(segments[1].highlight.is_some());
    }
}
