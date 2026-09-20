//! Maps to: CC `components/messages/RateLimitMessage.tsx`.

use crate::components::message_response::MessageResponse;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct RateLimitMessageProps {
    pub text: String,
    pub hint: String,
    pub add_margin: bool,
}

#[component]
pub fn RateLimitMessage(
    props: &RateLimitMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let text = if props.hint.is_empty() {
        props.text.clone()
    } else {
        format!("{} · {}", props.text, props.hint)
    };

    element! {
        MessageResponse(content: text, color: Some(theme.error))
    }
}
