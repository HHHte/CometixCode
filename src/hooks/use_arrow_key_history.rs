//! Maps to: CC `hooks/useArrowKeyHistory.tsx` (prompt subset).
//! Up/Down history navigation is intentionally outside `use_text_input`: the
//! text hook only reports cursor fallthrough; this hook owns draft preservation,
//! cache loading, mode-prefixed display values, and pasted-content restoration.

use crate::components::prompt_input::input_paste::PastedContent;
use crate::hooks::use_text_input::HistoryDirection;
use crate::utils::prompt_history::{self, HistoryEntry};
use iocraft::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct ArrowKeyHistoryState {
    pub history_index: State<usize>,
    saved_draft: State<Option<(String, BTreeMap<usize, PastedContent>)>>,
    history_cache: State<Vec<HistoryEntry>>,
}

impl ArrowKeyHistoryState {
    /// Navigate history. Returns `true` only when navigation reaches a boundary;
    /// PromptInput uses Down-at-bottom to enter its selectable footer.
    pub fn navigate(
        &mut self,
        direction: HistoryDirection,
        input: State<String>,
        cursor_offset: State<usize>,
        pasted_contents: State<BTreeMap<usize, PastedContent>>,
    ) -> bool {
        match direction {
            HistoryDirection::Up => self.on_history_up(input, cursor_offset, pasted_contents),
            HistoryDirection::Down => self.on_history_down(input, cursor_offset, pasted_contents),
        }
    }

    pub fn reset(&mut self) {
        self.history_index.set(0);
        self.saved_draft.set(None);
        self.history_cache.set(Vec::new());
    }

    fn on_history_up(
        &mut self,
        mut input: State<String>,
        mut cursor_offset: State<usize>,
        mut pasted_contents: State<BTreeMap<usize, PastedContent>>,
    ) -> bool {
        let target_index = self.history_index.get();
        if target_index == 0 {
            let draft = input.read().clone();
            self.saved_draft
                .set((!draft.trim().is_empty()).then(|| (draft, pasted_contents.read().clone())));
        }

        let mut cache = self.history_cache.read().clone();
        if cache.len() <= target_index {
            cache = prompt_history::get_history();
            self.history_cache.set(cache.clone());
        }

        let Some(entry) = cache.get(target_index).cloned() else {
            return true;
        };

        self.history_index.set(target_index + 1);
        // CC places the cursor at the start when moving up into a history item.
        cursor_offset.set(0);
        pasted_contents.set(entry.pasted_contents);
        input.set(entry.display);
        false
    }

    fn on_history_down(
        &mut self,
        mut input: State<String>,
        mut cursor_offset: State<usize>,
        mut pasted_contents: State<BTreeMap<usize, PastedContent>>,
    ) -> bool {
        let current_index = self.history_index.get();
        if current_index > 1 {
            let cache = self.history_cache.read();
            if let Some(entry) = cache.get(current_index - 2).cloned() {
                self.history_index.set(current_index - 1);
                cursor_offset.set(entry.display.len());
                pasted_contents.set(entry.pasted_contents);
                input.set(entry.display);
            }
            false
        } else if current_index == 1 {
            self.history_index.set(0);
            let (value, pasted) = self
                .saved_draft
                .read()
                .clone()
                .unwrap_or_else(|| (String::new(), BTreeMap::new()));
            cursor_offset.set(value.len());
            pasted_contents.set(pasted);
            input.set(value);
            false
        } else {
            true
        }
    }
}

pub fn use_arrow_key_history(hooks: &mut Hooks) -> ArrowKeyHistoryState {
    ArrowKeyHistoryState {
        history_index: hooks.use_state(|| 0usize),
        saved_draft: hooks.use_state(|| Option::<(String, BTreeMap<usize, PastedContent>)>::None),
        history_cache: hooks.use_state(Vec::<HistoryEntry>::new),
    }
}
