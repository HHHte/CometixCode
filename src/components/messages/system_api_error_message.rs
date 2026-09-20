//! Maps to: CC `components/messages/SystemAPIErrorMessage.tsx`.

use crate::components::message_response::MessageResponse;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct SystemApiErrorMessageProps {
    pub error: String,
    pub retry_attempt: u32,
    pub max_retries: u32,
    pub retry_in_seconds: u32,
    pub add_margin: bool,
}

#[component]
pub fn SystemApiErrorMessage(
    props: &SystemApiErrorMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let text = format!(
        "{} · Retrying in {}s (attempt {}/{})",
        props.error, props.retry_in_seconds, props.retry_attempt, props.max_retries
    );

    element! {
        MessageResponse(content: text, color: Some(theme.error))
    }
}
