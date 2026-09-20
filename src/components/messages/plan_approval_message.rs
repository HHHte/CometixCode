//! Maps to: CC `components/messages/PlanApprovalMessage.tsx`.

use crate::components::message_response::MessageResponse;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct PlanApprovalMessageProps {
    pub title: String,
    pub approved: bool,
    pub add_margin: bool,
}

#[component]
pub fn PlanApprovalMessage(
    props: &PlanApprovalMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let label = if props.approved {
        "approved"
    } else {
        "needs approval"
    };
    let color = if props.approved {
        theme.success
    } else {
        theme.warning
    };

    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            MessageResponse(content: format!("Plan {}: {}", label, props.title), color: Some(color))
        }
    }
}
