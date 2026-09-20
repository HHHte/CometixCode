//! Maps to: CC `components/messages/UserBashOutputMessage.tsx`.

use crate::components::message_response::MessageResponse;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserBashOutputMessageProps {
    pub output: String,
    pub is_error: bool,
    pub add_margin: bool,
}

#[component]
pub fn UserBashOutputMessage(
    props: &UserBashOutputMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let is_empty = props.output.trim().is_empty();
    let color = if props.is_error {
        Some(theme.error)
    } else if is_empty {
        Some(theme.inactive)
    } else {
        None
    };
    let output = if is_empty {
        "(No output)".to_string()
    } else {
        props.output.clone()
    };

    element! {
        MessageResponse(content: output, color: color, use_default_color: !props.is_error && !is_empty)
    }
}
