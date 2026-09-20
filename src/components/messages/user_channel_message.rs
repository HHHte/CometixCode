//! Maps to: CC `components/messages/UserChannelMessage.tsx`.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserChannelMessageProps {
    pub source: String,
    pub content: String,
    pub add_margin: bool,
}

#[component]
pub fn UserChannelMessage(props: &UserChannelMessageProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            MessageResponse(content: format!("{}: {}", props.source, props.content))
        }
    }
}
