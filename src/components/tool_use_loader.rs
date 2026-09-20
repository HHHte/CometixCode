//! Maps to: CC `components/ToolUseLoader.tsx`.
//!
//! Fixed-width status dot. Unresolved rows use dim default foreground (not
//! `theme.inactive`) and may blink via `use_blink` / `use_terminal_focus`.
//! The dim/bold split from the following tool name is intentional — same
//! chalk/ANSI reset hazard documented upstream.

use crate::constants::figures::BLACK_CIRCLE;
use crate::hooks::use_blink::use_blink;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ToolUseLoaderProps {
    pub is_error: bool,
    pub is_unresolved: bool,
    pub should_animate: bool,
}

#[component]
pub fn ToolUseLoader(
    props: &ToolUseLoaderProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    // CC: `const [ref, isBlinking] = useBlink(shouldAnimate)`
    let is_blinking = use_blink(&mut hooks, props.should_animate);

    // CC: `isUnresolved ? undefined : isError ? 'error' : 'success'`
    let color = if props.is_unresolved {
        None
    } else if props.is_error {
        Some(theme.error)
    } else {
        Some(theme.success)
    };

    // CC: `!shouldAnimate || isBlinking || isError || !isUnresolved ? ● : ' '`
    let glyph = if tool_use_loader_glyph_visible(
        props.should_animate,
        is_blinking,
        props.is_error,
        props.is_unresolved,
    ) {
        BLACK_CIRCLE
    } else {
        " "
    };

    element! {
        View(min_width: 2u32, flex_shrink: 0.0f32) {
            Text(
                content: glyph.to_string(),
                color: color,
                dim: props.is_unresolved,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}

/// Maps to: CC ToolUseLoader glyph ternary.
pub(crate) fn tool_use_loader_glyph_visible(
    should_animate: bool,
    is_blinking: bool,
    is_error: bool,
    is_unresolved: bool,
) -> bool {
    !should_animate || is_blinking || is_error || !is_unresolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::use_blink::BLINK_INTERVAL_MS;

    #[test]
    fn tool_use_loader_glyph_matches_official_ternary() {
        assert!(tool_use_loader_glyph_visible(false, false, false, true));
        assert!(tool_use_loader_glyph_visible(true, true, false, true));
        assert!(!tool_use_loader_glyph_visible(true, false, false, true));
        assert!(tool_use_loader_glyph_visible(true, false, true, true));
        assert!(tool_use_loader_glyph_visible(true, false, false, false));
    }

    #[test]
    fn tool_use_loader_keeps_fixed_two_column_status_cell_with_dim_unresolved() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                View(flex_direction: FlexDirection::Row) {
                    ToolUseLoader(is_error: false, is_unresolved: true, should_animate: false)
                    Text(content: "Bash".to_string(), weight: Weight::Bold)
                }
            }
        }
        .render(None);

        assert_eq!(
            canvas.to_string().trim_end(),
            format!("{} Bash", BLACK_CIRCLE)
        );
        let dot_style = canvas.resolved_text_style(0, 0).expect("dot style");
        // Official: color undefined + dimColor=true (not theme.inactive).
        assert_eq!(dot_style.color, None);
        assert!(dot_style.dim);
        assert_eq!(dot_style.weight, Weight::Light);
        let title_style = canvas.resolved_text_style(2, 0).expect("title style");
        assert_eq!(title_style.color, None);
        assert_eq!(title_style.weight, Weight::Bold);
        // The tool name sits immediately after the dim dot. CC warns in
        // `ToolUseLoader.tsx:21-27` that this pairing renders the name dimmed
        // when the intensity is not cleared between them — which is what a
        // tool awaiting permission looked like.
        assert!(!title_style.dim);
        let _ = BLINK_INTERVAL_MS;
    }

    #[test]
    fn tool_use_loader_resolved_success_uses_theme_success_without_dim() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                ToolUseLoader(is_error: false, is_unresolved: false, should_animate: false)
            }
        }
        .render(None);
        let theme = *crate::utils::theme::current();
        let dot_style = canvas.resolved_text_style(0, 0).expect("dot style");
        assert_eq!(dot_style.color, Some(theme.success));
        assert_eq!(dot_style.weight, Weight::Normal);
    }
}
