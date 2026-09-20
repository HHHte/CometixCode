//! Maps to: CC `components/messages/UserLocalCommandOutputMessage.tsx`.

use crate::components::markdown::Markdown;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserLocalCommandOutputMessageProps {
    pub output: String,
    pub is_error: bool,
    pub add_margin: bool,
}

#[component]
pub fn UserLocalCommandOutputMessage(
    props: &UserLocalCommandOutputMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let is_empty = props.output.trim().is_empty();
    let output = if is_empty {
        "(no content)".to_string()
    } else {
        props.output.trim().to_string()
    };

    element! {
        View(
            flex_direction: FlexDirection::Row,
        ) {
            Text(content: "  ⎿  ".to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
            View(flex_direction: FlexDirection::Column, flex_grow: 1.0f32) {
                #(if is_empty {
                    Some(element! { Text(content: output, color: theme.inactive) }.into_any())
                } else {
                    Some(element! { Markdown(content: output) }.into_any())
                })
            }
        }
    }
}
