//! Maps to: CC `components/design-system/ProgressBar.tsx`.
//! Draws a fixed-width terminal progress bar with eighth-block precision.

use iocraft::prelude::*;

const BLOCKS: [&str; 9] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];

pub fn progress_bar_text(input_ratio: f32, width: usize) -> String {
    let ratio = input_ratio.clamp(0.0, 1.0);
    let whole = (ratio * width as f32).floor() as usize;
    let mut segments = vec![BLOCKS[BLOCKS.len() - 1].repeat(whole)];

    if whole < width {
        let remainder = ratio * width as f32 - whole as f32;
        let middle = (remainder * BLOCKS.len() as f32).floor() as usize;
        segments.push(BLOCKS[middle.min(BLOCKS.len() - 1)].to_string());

        let empty = width.saturating_sub(whole + 1);
        if empty > 0 {
            segments.push(BLOCKS[0].repeat(empty));
        }
    }

    segments.join("")
}

#[derive(Default, Props)]
pub struct ProgressBarProps {
    pub ratio: f32,
    pub width: u32,
    pub fill_color: Option<Color>,
    pub empty_color: Option<Color>,
}

#[component]
pub fn ProgressBar(props: &ProgressBarProps) -> impl Into<AnyElement<'static>> {
    element! {
        Text(
            content: progress_bar_text(props.ratio, props.width as usize),
            color: props.fill_color,
            background_color: props.empty_color,
            wrap: TextWrap::NoWrap,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_clamps_ratio_and_matches_official_segments() {
        assert_eq!(progress_bar_text(-1.0, 4), "    ");
        assert_eq!(progress_bar_text(1.0, 4), "████");
        assert_eq!(progress_bar_text(0.5, 4), "██  ");
        assert_eq!(progress_bar_text(0.125, 4), "▌   ");
    }

    #[test]
    fn progress_bar_applies_fill_and_empty_colors() {
        let canvas = element! {
            ProgressBar(
                ratio: 0.5f32,
                width: 4u32,
                fill_color: Some(Color::Green),
                empty_color: Some(Color::DarkGrey),
            )
        }
        .render(Some(10));
        let first_cell = canvas.cell(0, 0).expect("progress cell");

        assert_eq!(canvas.get_text(0, 0, 4, 1), "██  ");
        assert_eq!(
            first_cell.text_style().and_then(|style| style.color),
            Some(Color::Green)
        );
        assert_eq!(first_cell.background_color, Some(Color::DarkGrey));
    }
}
