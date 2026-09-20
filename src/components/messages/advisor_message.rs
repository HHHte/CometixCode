//! Maps to: CC `components/messages/AdvisorMessage.tsx`.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct AdvisorMessageProps {
    pub content: String,
    pub add_margin: bool,
}

#[component]
pub fn AdvisorMessage(props: &AdvisorMessageProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            MessageResponse(content: format!("Advisor: {}", props.content))
        }
    }
}
