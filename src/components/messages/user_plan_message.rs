//! Maps to: CC `components/messages/UserPlanMessage.tsx`.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserPlanMessageProps {
    pub content: String,
    pub add_margin: bool,
}

#[component]
pub fn UserPlanMessage(
    props: &UserPlanMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
            border_style: BorderStyle::Round,
            border_color: theme.plan_mode,
            padding_left: 1u32,
            padding_right: 1u32,
        ) {
            Text(content: "Plan", color: theme.plan_mode, weight: Weight::Bold, wrap: TextWrap::NoWrap)
            Text(content: props.content.clone())
        }
    }
}
