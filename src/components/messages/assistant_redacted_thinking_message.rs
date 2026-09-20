//! Maps to: CC `components/messages/AssistantRedactedThinkingMessage.tsx`.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct AssistantRedactedThinkingMessageProps {
    pub add_margin: bool,
}

#[component]
pub fn AssistantRedactedThinkingMessage(
    props: &AssistantRedactedThinkingMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();

    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            Text(content: "✻ Thinking…", color: theme.inactive, italic: true, wrap: TextWrap::NoWrap)
        }
    }
}
