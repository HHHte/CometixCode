//! Maps to: CC `components/messages/UserResourceUpdateMessage.tsx`.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserResourceUpdateMessageProps {
    pub content: String,
    pub add_margin: bool,
}

#[component]
pub fn UserResourceUpdateMessage(
    props: &UserResourceUpdateMessageProps,
) -> impl Into<AnyElement<'static>> {
    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            MessageResponse(content: format!("Resource update: {}", props.content))
        }
    }
}
