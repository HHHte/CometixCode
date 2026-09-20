//! Maps to: CC `components/messages/CompactBoundaryMessage.tsx`.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct CompactBoundaryMessageProps {
    pub add_margin: bool,
}

#[component]
pub fn CompactBoundaryMessage(
    props: &CompactBoundaryMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();

    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }, margin_bottom: 1u32) {
            Text(content: "✻ Conversation compacted (Ctrl+O for history)", color: theme.inactive, wrap: TextWrap::NoWrap)
        }
    }
}
