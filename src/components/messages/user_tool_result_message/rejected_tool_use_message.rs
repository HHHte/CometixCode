//! Maps to: CC `components/messages/UserToolResultMessage/RejectedToolUseMessage.tsx`.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct RejectedToolUseMessageProps {}

/// Maps to: CC
/// `components/messages/UserToolResultMessage/RejectedToolUseMessage.tsx:5-11`
/// `RejectedToolUseMessage`.
#[component]
pub fn RejectedToolUseMessage(
    _props: &RejectedToolUseMessageProps,
) -> impl Into<AnyElement<'static>> {
    element! {
        MessageResponse {
            Text(content: "Tool use rejected".to_string(), dim: true)
        }
    }
}
