//! Maps to: CC components/design-system/Pane.tsx
//! Main-screen Pane shape:
//!   <Box flexDirection="column" paddingTop={1}>
//!     <Divider color={color} />
//!     <Box flexDirection="column" paddingX={2}>{children}</Box>
//!   </Box>
//! Fullscreen modal shape:
//!   <Box flexDirection="column" paddingX={1} flexShrink={0}>{children}</Box>
//! because `FullscreenLayout` already draws the modal divider.

use super::divider::Divider;
use crate::context::modal_context::ModalContextSnapshot;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct PaneProps {
    pub color: Option<Color>,
    pub children: Vec<AnyElement<'static>>,
}

#[component]
pub fn Pane(props: &mut PaneProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let color = props.color;
    let inside_modal = hooks.try_use_context::<ModalContextSnapshot>().is_some();

    if inside_modal {
        return element! {
            View(
                flex_direction: FlexDirection::Column,
                padding_left: 1u32,
                padding_right: 1u32,
                flex_shrink: 0.0f32,
            ) {
                #(props.children.drain(..))
            }
        }
        .into_any();
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            padding_top: 1u32,
            flex_shrink: 0.0f32,
            align_self: AlignSelf::FLEX_START,
        ) {
            Divider(color: color)
            View(
                flex_direction: FlexDirection::Column,
                padding_left: 2u32,
                padding_right: 2u32,
            ) {
                #(props.children.drain(..))
            }
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_main_screen_renders_divider_frame_like_official() {
        let canvas = element! {
            Pane(color: Some(Color::Blue)) {
                Text(content: "Pane body".to_string())
            }
        }
        .render(Some(24));
        let text = canvas.to_string();

        assert!(text.contains('─'), "canvas=\n{text}");
        assert!(text.contains("Pane body"), "canvas=\n{text}");
    }

    #[test]
    fn pane_inside_modal_skips_own_divider_like_official_modal_context() {
        let canvas = element! {
            ContextProvider(value: Context::owned(ModalContextSnapshot { rows: 10, columns: 20 })) {
                Pane(color: Some(Color::Blue)) {
                    Text(content: "Pane body".to_string())
                }
            }
        }
        .render(Some(24));
        let text = canvas.to_string();

        assert!(
            !text.contains('─'),
            "modal Pane should not double-frame; canvas=\n{text}"
        );
        assert!(text.contains("Pane body"), "canvas=\n{text}");
    }
}
