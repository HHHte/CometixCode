//! Maps to: CC `components/PromptInput/HistorySearchInput.tsx`.

use crate::components::text_input::TextInput;
use iocraft::prelude::*;
use unicode_width::UnicodeWidthStr;

#[derive(Default, Props)]
pub struct HistorySearchInputProps {
    pub value: Option<State<String>>,
    pub cursor_offset: Option<State<usize>>,
    pub history_failed_match: bool,
}

#[component]
pub fn HistorySearchInput(
    props: &HistorySearchInputProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let internal_value = hooks.use_state(String::new);
    let internal_cursor = hooks.use_state(|| 0usize);
    let value = props.value.unwrap_or(internal_value);
    let cursor = props.cursor_offset.unwrap_or(internal_cursor);
    let width = UnicodeWidthStr::width(value.read().as_str()) + 1;
    element! {
        View(flex_direction: FlexDirection::Row, column_gap: 1u32) {
            Text(
                content: if props.history_failed_match { "no matching prompt:" } else { "search prompts:" }.to_string(),
                dim: true,
                wrap: TextWrap::NoWrap,
            )
            TextInput(
                value: value,
                cursor_offset: cursor,
                columns: width,
                focus: true,
                show_cursor: true,
                multiline: false,
                dim_color: true,
            )
        }
    }
}
