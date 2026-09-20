//! Maps to: CC `components/design-system/KeyboardShortcutHint.tsx`.
//! Renders inline keyboard help such as `Enter to confirm`.

use iocraft::prelude::*;

/// iocraft projection of Ink parent `<Text>` style inheritance for rich hint
/// trees such as Dialog and wizard footers.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyboardShortcutHintStyleContext {
    pub dim: bool,
    pub italic: bool,
}

#[derive(Default, Props)]
pub struct KeyboardShortcutHintProps {
    pub shortcut: String,
    pub action: String,
    pub parens: bool,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
}

#[component]
pub fn KeyboardShortcutHint(
    props: &KeyboardShortcutHintProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let inherited = hooks
        .try_use_context::<KeyboardShortcutHintStyleContext>()
        .map(|context| *context)
        .unwrap_or_default();
    let dim = props.dim || inherited.dim;
    let italic = props.italic || inherited.italic;
    element! {
        View(flex_direction: FlexDirection::Row, flex_shrink: 0.0f32) {
            #(if props.parens {
                Some(element! { Text(content: "(".to_string(), dim: dim, italic: italic, wrap: TextWrap::NoWrap) })
            } else {
                None
            })
            Text(
                content: props.shortcut.clone(),
                weight: if props.bold { Weight::Bold } else { Weight::Normal },
                dim: dim,
                italic: italic,
                wrap: TextWrap::NoWrap,
            )
            Text(content: format!(" to {}", props.action), dim: dim, italic: italic, wrap: TextWrap::NoWrap)
            #(if props.parens {
                Some(element! { Text(content: ")".to_string(), dim: dim, italic: italic, wrap: TextWrap::NoWrap) })
            } else {
                None
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_shortcut_hint_matches_official_text_shape() {
        let canvas = element! {
            KeyboardShortcutHint(
                shortcut: "Enter".to_string(),
                action: "confirm".to_string(),
                bold: true,
            )
        }
        .render(Some(40));

        assert_eq!(canvas.to_string().trim_end(), "Enter to confirm");
        assert_eq!(
            canvas
                .resolved_text_style(0, 0)
                .expect("shortcut style")
                .weight,
            Weight::Bold
        );
    }

    #[test]
    fn keyboard_shortcut_hint_inherits_parent_text_projection() {
        let canvas = element! {
            ContextProvider(value: Context::owned(KeyboardShortcutHintStyleContext {
                dim: true,
                italic: true,
            })) {
                KeyboardShortcutHint(
                    shortcut: "Esc".to_string(),
                    action: "back".to_string(),
                )
            }
        }
        .render(Some(40));
        let style = canvas.resolved_text_style(0, 0).expect("hint style");
        assert_eq!(style.weight, Weight::Light);
        assert!(style.italic);
    }

    #[test]
    fn keyboard_shortcut_hint_supports_parentheses() {
        let canvas = element! {
            KeyboardShortcutHint(
                shortcut: "Esc".to_string(),
                action: "cancel".to_string(),
                parens: true,
            )
        }
        .render(Some(40));

        assert_eq!(canvas.to_string().trim_end(), "(Esc to cancel)");
    }
}
