//! Maps to: CC `components/PromptInput/useMaybeTruncateInput.ts:1-55`.

use super::input_paste::{PastedContent, TRUNCATION_THRESHOLD, maybe_truncate_input};
use iocraft::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub struct MaybeTruncateInputState {
    pub input: State<String>,
    pub cursor_offset: State<usize>,
    pub pasted_contents: State<BTreeMap<usize, PastedContent>>,
}

/// Applies the official once-per-input truncation effect and resets its guard
/// after submission clears the input.
pub fn use_maybe_truncate_input(hooks: &mut Hooks, state: MaybeTruncateInputState) -> bool {
    let mut applied = hooks.use_state(|| false);
    let input_value = state.input.to_string();
    if input_value.is_empty() {
        if applied.get() {
            applied.set(false);
        }
        return false;
    }
    if !applied.get() && input_value.encode_utf16().count() > TRUNCATION_THRESHOLD {
        let (new_input, new_contents) =
            maybe_truncate_input(&input_value, &state.pasted_contents.read());
        // Rust TextInput offsets are UTF-8 byte offsets; use its native unit
        // after preserving JS UTF-16 semantics inside the truncation helper.
        let new_offset = new_input.len();
        let mut input = state.input;
        let mut cursor = state.cursor_offset;
        let mut pasted = state.pasted_contents;
        input.set(new_input);
        cursor.set(new_offset);
        pasted.set(new_contents);
        applied.set(true);
        return true;
    }
    applied.get()
}
