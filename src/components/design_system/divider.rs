//! Maps to: CC components/design-system/Divider.tsx
//! CC Divider (line 66-97):
//!   - Takes optional width (defaults to terminal width), color, char, padding,
//!     and title.
//!   - effectiveWidth = max(0, (width ?? terminalWidth) - padding)
//!   - Renders: <Text color={color} dimColor={!color}>{char.repeat(effectiveWidth)}</Text>

use iocraft::prelude::*;
use regex::Regex;
use std::sync::LazyLock;
use unicode_width::UnicodeWidthStr;

static ANSI_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07]*(?:\x07|\x1b\\)").expect("valid ANSI regex")
});

fn visible_width(text: &str) -> usize {
    let stripped = ANSI_PATTERN.replace_all(text, "");
    UnicodeWidthStr::width(stripped.as_ref())
}

#[derive(Default, Props)]
pub struct DividerProps {
    pub color: Option<Color>,
    pub width: Option<u32>,
    /// Maps to CC's `char` prop.
    pub character: Option<String>,
    pub padding: u32,
    pub title: Option<String>,
}

#[component]
pub fn Divider(props: &DividerProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (term_width, _) = hooks.use_terminal_size();
    let color = props.color;
    let dim = props.color.is_none();
    let character = props.character.as_deref().unwrap_or("─");
    let effective_width = props
        .width
        .unwrap_or(term_width as u32)
        .saturating_sub(props.padding) as usize;

    let rendered: AnyElement<'static> = if let Some(title) = props.title.as_deref() {
        let title_width = visible_width(title) + 2;
        let side_width = effective_width.saturating_sub(title_width);
        let left_width = side_width / 2;
        let right_width = side_width - left_width;
        element! {
            View(flex_direction: FlexDirection::Row, flex_shrink: 0.0f32) {
                Text(content: format!("{} ", character.repeat(left_width)), color: color, dim: dim, wrap: TextWrap::NoWrap)
                // CC nests Ansi inside a dim Text. Carry that parent style
                // separately so an explicit ANSI bold span remains both bold
                // and dim without changing direct `Ansi dimColor` semantics.
                Ansi(content: title.to_string(), inherited_dim: true, wrap: TextWrap::NoWrap)
                Text(content: format!(" {}", character.repeat(right_width)), color: color, dim: dim, wrap: TextWrap::NoWrap)
            }
        }
        .into_any()
    } else {
        element! {
            Text(content: character.repeat(effective_width), color: color, dim: dim, wrap: TextWrap::NoWrap)
        }
        .into_any()
    };

    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_uses_explicit_width_and_padding_like_official() {
        let canvas = element! {
            Divider(width: 8u32, padding: 3u32)
        }
        .render(Some(20));

        assert_eq!(canvas.get_text(0, 0, 5, 1), "─────");
        assert_eq!(canvas.cell(5, 0).and_then(|cell| cell.text()), None);
    }

    #[test]
    fn divider_centers_title_like_official() {
        let canvas = element! {
            Divider(width: 12u32, title: "new".to_string())
        }
        .render(Some(20));

        assert_eq!(canvas.get_text(0, 0, 12, 1), "─── new ────");
    }

    #[test]
    fn divider_title_is_dimmed_and_ansi_width_is_ignored() {
        let canvas = element! {
            Divider(width: 12u32, title: "\u{1b}[31mnew\u{1b}[0m".to_string())
        }
        .render(Some(20));

        assert_eq!(canvas.get_text(0, 0, 12, 1), "─── new ────");
        let title = canvas.resolved_text_style(4, 0).expect("title style");
        assert_eq!(title.weight, Weight::Normal);
        assert!(title.dim);
        assert!(title.is_dim());
    }

    #[test]
    fn divider_inherited_dim_keeps_ansi_title_bold_like_official() {
        let canvas = element! {
            Divider(width: 8u32, title: "\u{1b}[1m10\u{1b}[22m".to_string())
        }
        .render(Some(20));

        assert_eq!(canvas.get_text(0, 0, 8, 1), "── 10 ──");
        let count = canvas.resolved_text_style(3, 0).expect("count style");
        assert_eq!(count.weight, Weight::Bold);
        assert!(count.dim);
        assert!(count.is_dim());
    }
}
