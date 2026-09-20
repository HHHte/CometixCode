//! Maps to: CC `components/messages/UserBashInputMessage.tsx`.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserBashInputMessageProps {
    pub command: String,
    pub add_margin: bool,
}

#[component]
pub fn UserBashInputMessage(
    props: &UserBashInputMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();

    element! {
        View(
            flex_direction: FlexDirection::Row,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
            background_color: theme.bash_message_bg,
            padding_right: 1u32,
        ) {
            Text(content: "! ", color: theme.bash_border, wrap: TextWrap::NoWrap)
            Text(content: props.command.clone(), color: theme.text)
        }
    }
}
