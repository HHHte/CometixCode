//! Maps to: CC `components/messages/UserImageMessage.tsx`.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserImageMessageProps {
    pub image_id: Option<u32>,
    pub add_margin: bool,
}

#[component]
pub fn UserImageMessage(props: &UserImageMessageProps) -> impl Into<AnyElement<'static>> {
    let label = match props.image_id {
        Some(id) => format!("[Image #{}]", id),
        None => "[Image]".to_string(),
    };

    if props.add_margin {
        return element! {
            View(margin_top: 1u32) {
                Text(content: label)
            }
        }
        .into_any();
    }

    element! {
        MessageResponse(content: label)
    }
    .into_any()
}
