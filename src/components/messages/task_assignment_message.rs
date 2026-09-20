//! Maps to: CC `components/messages/TaskAssignmentMessage.tsx`.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct TaskAssignmentMessageProps {
    pub task_id: String,
    pub subject: String,
    pub add_margin: bool,
}

#[component]
pub fn TaskAssignmentMessage(props: &TaskAssignmentMessageProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(margin_top: if props.add_margin { 1u32 } else { 0u32 }) {
            MessageResponse(content: format!("Task assigned: #{} - {}", props.task_id, props.subject))
        }
    }
}
