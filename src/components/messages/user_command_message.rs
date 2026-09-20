//! Maps to: CC `components/messages/UserCommandMessage.tsx`.

use super::user_prompt_message::UserPromptMessage;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserCommandMessageProps {
    pub command: String,
    pub args: String,
    pub add_margin: bool,
}

#[component]
pub fn UserCommandMessage(props: &UserCommandMessageProps) -> impl Into<AnyElement<'static>> {
    let content = if props.args.is_empty() {
        format!("/{}", props.command)
    } else {
        format!("/{} {}", props.command, props.args)
    };

    element! {
        UserPromptMessage(content: content, add_margin: props.add_margin)
    }
}
