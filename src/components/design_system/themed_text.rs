//! Maps to: CC `components/design-system/ThemedText.tsx`.
//! Theme-aware Text wrapper for Rust callers that already resolve theme keys to
//! `iocraft::Color`. Uncolored dim text uses the current theme's inactive color.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ThemedTextProps {
    pub content: String,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub dim_color: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
    pub wrap: Option<TextWrap>,
}

#[component]
pub fn ThemedText(props: &ThemedTextProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let resolved_color = if props.color.is_none() && props.dim_color {
        Some(theme.inactive)
    } else {
        props.color
    };

    element! {
        Text(
            content: props.content.clone(),
            color: resolved_color,
            background_color: props.background_color,
            weight: if props.bold { Weight::Bold } else { Weight::Normal },
            italic: props.italic,
            underline: props.underline,
            strikethrough: props.strikethrough,
            invert: props.inverse,
            wrap: props.wrap.unwrap_or(TextWrap::Wrap),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn themed_text_resolves_dim_and_style_props() {
        let current_theme = *theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(current_theme)) {
                ThemedText(content: "dim".to_string(), dim_color: true, bold: true)
            }
        }
        .render(Some(20));
        let style = canvas.resolved_text_style(0, 0).expect("text style");

        assert_eq!(canvas.to_string().trim_end(), "dim");
        assert_eq!(style.color, Some(current_theme.inactive));
        assert_eq!(style.weight, Weight::Bold);
    }
}
