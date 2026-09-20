//! Maps to: CC `components/messages/UserMemoryInputMessage.tsx`.

use crate::components::message_response::MessageResponse;
use crate::utils::messages::extract_tag;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct UserMemoryInputMessageProps {
    pub text: String,
    pub add_margin: bool,
}

#[component]
pub fn UserMemoryInputMessage(
    props: &UserMemoryInputMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let Some(input) = extract_tag(&props.text, "user-memory-input") else {
        return element! { View }.into_any();
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
            width: 100pct,
        ) {
            View(flex_direction: FlexDirection::Row) {
                Text(
                    content: "#".to_string(),
                    color: theme.remember,
                    background_color: theme.memory_bg,
                    wrap: TextWrap::NoWrap,
                )
                Text(
                    content: format!(" {input} "),
                    color: theme.text,
                    background_color: theme.memory_bg,
                )
            }
            MessageResponse(content: "Got it.".to_string(), color: Some(theme.inactive))
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_memory_input_extracts_tag_and_applies_memory_background_to_text_cells() {
        let theme = *crate::utils::theme::current();
        let canvas = element! {
            ContextProvider(value: Context::owned(theme)) {
                UserMemoryInputMessage(
                    text: "prefix <user-memory-input>Use Chinese replies.</user-memory-input>".to_string(),
                    add_margin: false,
                )
            }
        }
        .render(None);

        let rendered = canvas.to_string();
        assert!(rendered.contains("# Use Chinese replies."));
        assert!(rendered.contains("Got it."));
        assert_eq!(
            canvas.cell(0, 0).unwrap().background_color,
            Some(theme.memory_bg)
        );
        assert_eq!(
            canvas.cell(1, 0).unwrap().background_color,
            Some(theme.memory_bg)
        );
    }

    #[test]
    fn user_memory_input_without_tag_renders_nothing_like_official() {
        let canvas = element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                UserMemoryInputMessage(text: "Use Chinese replies.".to_string())
            }
        }
        .render(None);

        assert_eq!(canvas.to_string(), "");
    }
}
