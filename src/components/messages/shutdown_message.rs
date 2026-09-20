//! Maps to: CC `components/messages/ShutdownMessage.tsx`.

use crate::components::message_response::MessageResponse;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ShutdownMessageProps {
    pub text: String,
    pub add_margin: bool,
}

#[component]
pub fn ShutdownMessage(
    props: &ShutdownMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();

    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            MessageResponse(content: props.text.clone(), color: Some(theme.warning))
        }
    }
}
