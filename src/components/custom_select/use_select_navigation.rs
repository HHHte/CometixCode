//! Maps to: CC `components/CustomSelect/use-select-navigation.ts`.
//!
//! The reducer is ported 1:1 as methods on `SelectNavigationState` (CC
//! actions: focus-next-option / focus-previous-option / focus-next-page /
//! focus-previous-page / set-focus / reset). The hook wraps the state in an
//! iocraft `State` and mirrors CC's effects: reset when the option values
//! change (preserving the viewport), programmatic `focus_value`, and a
//! focus-change signal in place of the `onFocus` effect (the caller reads it
//! synchronously in the component body).

use super::option_map::OptionMap;
use iocraft::prelude::*;

/// Maps to: CC use-select-navigation.ts `State<T>`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectNavigationState {
    pub option_map: OptionMap,
    pub visible_option_count: usize,
    pub focused_value: Option<String>,
    pub visible_from_index: usize,
    pub visible_to_index: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Viewport {
    pub visible_from_index: usize,
    pub visible_to_index: usize,
}

impl SelectNavigationState {
    /// Maps to: CC `createDefaultState` (:424-503).
    pub fn create_default(
        visible_option_count: Option<usize>,
        values: &[String],
        initial_focus_value: Option<&str>,
        current_viewport: Option<Viewport>,
    ) -> Self {
        // CC :433-436 — an explicit count clamps to the option count; no
        // count means "show everything".
        let visible_option_count = match visible_option_count {
            Some(count) => count.min(values.len()),
            None => values.len(),
        };
        let option_map = OptionMap::new(values.iter().cloned());
        let focused_item_index = initial_focus_value
            .and_then(|value| option_map.get(value))
            .map(|item| item.index);
        let focused_value = match focused_item_index {
            Some(index) => option_map.item_at(index).map(|item| item.value.clone()),
            None => option_map.first().map(|item| item.value.clone()),
        };

        let mut visible_from_index = 0usize;
        let mut visible_to_index = visible_option_count;

        if let Some(focused_index) = focused_item_index {
            if let Some(viewport) = current_viewport {
                if focused_index >= viewport.visible_from_index
                    && focused_index < viewport.visible_to_index
                {
                    // CC :450-461 — keep the previous viewport when the
                    // focused item is still inside it.
                    visible_from_index = viewport.visible_from_index;
                    visible_to_index = option_map.size().min(viewport.visible_to_index);
                } else if focused_index < viewport.visible_from_index {
                    // CC :465-471 — scroll up, item at the top.
                    visible_from_index = focused_index;
                    visible_to_index = option_map
                        .size()
                        .min(visible_from_index + visible_option_count);
                } else {
                    // CC :472-476 — scroll down, item at the bottom.
                    visible_to_index = option_map.size().min(focused_index + 1);
                    visible_from_index = visible_to_index.saturating_sub(visible_option_count);
                }
            } else if focused_index >= visible_option_count {
                // CC :478-483 — no viewport yet; show the item at the bottom.
                visible_to_index = option_map.size().min(focused_index + 1);
                visible_from_index = visible_to_index.saturating_sub(visible_option_count);
            }

            // CC :485-493 — clamp viewport bounds.
            visible_from_index = visible_from_index.min(option_map.size().saturating_sub(1));
            visible_to_index = option_map
                .size()
                .min(visible_to_index.max(visible_option_count));
        }

        Self {
            option_map,
            visible_option_count,
            focused_value,
            visible_from_index,
            visible_to_index,
        }
    }

    /// Maps to: CC reducer `focus-next-option` (:76-126) — wraps to the
    /// first item (resetting the viewport) past the end.
    pub fn focus_next_option(&mut self) {
        let Some(item) = self
            .focused_value
            .as_deref()
            .and_then(|value| self.option_map.get(value))
        else {
            return;
        };
        let wrapped = item.next.is_none();
        let next_index = match item.next {
            Some(index) => index,
            None => match self.option_map.first() {
                Some(first) => first.index,
                None => return,
            },
        };
        let next_value = match self.option_map.item_at(next_index) {
            Some(next) => next.value.clone(),
            None => return,
        };
        if wrapped {
            self.focused_value = Some(next_value);
            self.visible_from_index = 0;
            self.visible_to_index = self.visible_option_count;
            return;
        }
        self.focused_value = Some(next_value);
        if next_index >= self.visible_to_index {
            self.visible_to_index = self.option_map.size().min(self.visible_to_index + 1);
            self.visible_from_index = self.visible_to_index - self.visible_option_count;
        }
    }

    /// Maps to: CC reducer `focus-previous-option` (:128-180) — wraps to the
    /// last item (viewport pinned to the end) past the start.
    pub fn focus_previous_option(&mut self) {
        let Some(item) = self
            .focused_value
            .as_deref()
            .and_then(|value| self.option_map.get(value))
        else {
            return;
        };
        let wrapped = item.previous.is_none();
        let previous_index = match item.previous {
            Some(index) => index,
            None => match self.option_map.last() {
                Some(last) => last.index,
                None => return,
            },
        };
        let previous_value = match self.option_map.item_at(previous_index) {
            Some(previous) => previous.value.clone(),
            None => return,
        };
        if wrapped {
            self.focused_value = Some(previous_value);
            self.visible_to_index = self.option_map.size();
            self.visible_from_index = self
                .visible_to_index
                .saturating_sub(self.visible_option_count);
            return;
        }
        self.focused_value = Some(previous_value);
        if previous_index <= self.visible_from_index {
            self.visible_from_index = self.visible_from_index.saturating_sub(1);
            self.visible_to_index = self.visible_from_index + self.visible_option_count;
        }
    }

    /// Maps to: CC reducer `focus-next-page` (:182-229).
    pub fn focus_next_page(&mut self) {
        let Some(item) = self
            .focused_value
            .as_deref()
            .and_then(|value| self.option_map.get(value))
        else {
            return;
        };
        let target_index = self
            .option_map
            .size()
            .saturating_sub(1)
            .min(item.index + self.visible_option_count);
        let Some(target) = self.option_map.item_at(target_index) else {
            return;
        };
        self.focused_value = Some(target.value.clone());
        self.visible_to_index = self.option_map.size().min(target_index + 1);
        self.visible_from_index = self
            .visible_to_index
            .saturating_sub(self.visible_option_count);
    }

    /// Maps to: CC reducer `focus-previous-page` (:231-272).
    pub fn focus_previous_page(&mut self) {
        let Some(item) = self
            .focused_value
            .as_deref()
            .and_then(|value| self.option_map.get(value))
        else {
            return;
        };
        let target_index = item.index.saturating_sub(self.visible_option_count);
        let Some(target) = self.option_map.item_at(target_index) else {
            return;
        };
        self.focused_value = Some(target.value.clone());
        self.visible_from_index = target_index;
        self.visible_to_index = self
            .option_map
            .size()
            .min(self.visible_from_index + self.visible_option_count);
    }

    /// Maps to: CC reducer `set-focus` (:278-328) — minimal scrolling puts
    /// an out-of-view item at the nearest viewport edge.
    pub fn set_focus(&mut self, value: &str) {
        if self.focused_value.as_deref() == Some(value) {
            return;
        }
        let Some(item) = self.option_map.get(value) else {
            return;
        };
        let index = item.index;
        self.focused_value = Some(item.value.clone());
        if index >= self.visible_from_index && index < self.visible_to_index {
            return;
        }
        if index < self.visible_from_index {
            self.visible_from_index = index;
            self.visible_to_index = self
                .option_map
                .size()
                .min(self.visible_from_index + self.visible_option_count);
        } else {
            self.visible_to_index = self.option_map.size().min(index + 1);
            self.visible_from_index = self
                .visible_to_index
                .saturating_sub(self.visible_option_count);
        }
    }

    /// Maps to: CC `validatedFocusedValue` (:592-602) — falls back to the
    /// first option when the focused value no longer exists.
    pub fn validated_focused_value(&self) -> Option<String> {
        match self.focused_value.as_deref() {
            Some(value) if self.option_map.get(value).is_some() => Some(value.to_string()),
            _ => self.option_map.first().map(|item| item.value.clone()),
        }
    }

    /// 0-based index of the (validated) focused option.
    pub fn focused_index(&self) -> Option<usize> {
        self.validated_focused_value()
            .as_deref()
            .and_then(|value| self.option_map.get(value))
            .map(|item| item.index)
    }
}

pub struct UseSelectNavigationProps {
    /// Maps to: CC `visibleOptionCount` (None = show all options).
    pub visible_option_count: Option<usize>,
    /// Option values in display order (labels stay on the caller's data).
    pub values: Vec<String>,
    /// Maps to: CC `initialFocusValue`.
    pub initial_focus_value: Option<String>,
    /// Maps to: CC `focusValue` — programmatic focus from the parent.
    pub focus_value: Option<String>,
}

/// Maps to: CC `SelectNavigation` — a copyable handle over the navigation
/// state plus the focus-change signal that replaces the onFocus effect.
#[derive(Clone, Copy)]
pub struct SelectNavigation {
    pub state: State<SelectNavigationState>,
    last_notified_focus: State<Option<String>>,
}

impl SelectNavigation {
    pub fn snapshot(&self) -> SelectNavigationState {
        self.state.read().clone()
    }

    pub fn focus_next_option(&self) {
        let mut state = self.state;
        let mut next = state.read().clone();
        next.focus_next_option();
        state.set(next);
    }

    pub fn focus_previous_option(&self) {
        let mut state = self.state;
        let mut next = state.read().clone();
        next.focus_previous_option();
        state.set(next);
    }

    pub fn focus_next_page(&self) {
        let mut state = self.state;
        let mut next = state.read().clone();
        next.focus_next_page();
        state.set(next);
    }

    pub fn focus_previous_page(&self) {
        let mut state = self.state;
        let mut next = state.read().clone();
        next.focus_previous_page();
        state.set(next);
    }

    /// Maps to: CC `focusOption`.
    pub fn focus_option(&self, value: &str) {
        let mut state = self.state;
        let mut next = state.read().clone();
        next.set_focus(value);
        state.set(next);
    }

    pub fn focused_value(&self) -> Option<String> {
        self.state.read().validated_focused_value()
    }

    /// Maps to: CC onFocus effect (:614-618) — returns the newly focused
    /// value exactly once per change; the caller invokes its callback with
    /// it in the component body.
    pub fn take_focus_change(&self) -> Option<String> {
        let current = self.focused_value();
        if *self.last_notified_focus.read() == current {
            return None;
        }
        let mut last = self.last_notified_focus;
        last.set(current.clone());
        current
    }
}

/// Maps to: CC `useSelectNavigation`.
pub fn use_select_navigation(
    hooks: &mut Hooks,
    props: UseSelectNavigationProps,
) -> SelectNavigation {
    let state = hooks.use_state({
        let values = props.values.clone();
        let initial_focus = props
            .focus_value
            .clone()
            .or_else(|| props.initial_focus_value.clone());
        let visible_option_count = props.visible_option_count;
        move || {
            SelectNavigationState::create_default(
                visible_option_count,
                &values,
                initial_focus.as_deref(),
                None,
            )
        }
    });
    let mut last_values = hooks.use_state({
        let values = props.values.clone();
        move || values
    });
    let mut last_focus_value = hooks.use_state({
        let focus_value = props.focus_value.clone();
        move || focus_value
    });
    let last_notified_focus = hooks.use_state(|| Option::<String>::None);
    let navigation = SelectNavigation {
        state,
        last_notified_focus,
    };

    // CC :526-544 — options changed: reset, preserving the viewport and the
    // current focus when still valid.
    if *last_values.read() != props.values {
        let current = state.read().clone();
        let initial_focus = props
            .focus_value
            .clone()
            .or(current.focused_value.clone())
            .or_else(|| props.initial_focus_value.clone());
        let next = SelectNavigationState::create_default(
            props.visible_option_count,
            &props.values,
            initial_focus.as_deref(),
            Some(Viewport {
                visible_from_index: current.visible_from_index,
                visible_to_index: current.visible_to_index,
            }),
        );
        let mut state = state;
        state.set(next);
        last_values.set(props.values.clone());
    }

    // CC :620+ — programmatic focusValue changes focus the option.
    if *last_focus_value.read() != props.focus_value {
        if let Some(value) = props.focus_value.as_deref() {
            navigation.focus_option(value);
        }
        last_focus_value.set(props.focus_value.clone());
    }

    navigation
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    fn state(count: usize, names: &[&str], focus: Option<&str>) -> SelectNavigationState {
        SelectNavigationState::create_default(Some(count), &values(names), focus, None)
    }

    #[test]
    fn default_state_matches_official_initial_viewport() {
        let nav = state(5, &["a", "b", "c"], None);
        assert_eq!(nav.focused_value.as_deref(), Some("a"));
        assert_eq!(nav.visible_option_count, 3, "count clamps to option len");
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (0, 3));

        // Focus beyond the default viewport lands at the bottom (CC :478-483).
        let nav = state(2, &["a", "b", "c", "d"], Some("d"));
        assert_eq!(nav.focused_value.as_deref(), Some("d"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (2, 4));
    }

    #[test]
    fn focus_next_wraps_to_first_and_resets_viewport() {
        let mut nav = state(2, &["a", "b", "c"], Some("c"));
        nav.focus_next_option();
        assert_eq!(nav.focused_value.as_deref(), Some("a"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (0, 2));
    }

    #[test]
    fn focus_previous_wraps_to_last_and_pins_viewport_to_end() {
        let mut nav = state(2, &["a", "b", "c"], Some("a"));
        nav.focus_previous_option();
        assert_eq!(nav.focused_value.as_deref(), Some("c"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (1, 3));
    }

    #[test]
    fn focus_next_scrolls_window_by_one_like_official() {
        let mut nav = state(2, &["a", "b", "c", "d"], None);
        nav.focus_next_option();
        assert_eq!(nav.focused_value.as_deref(), Some("b"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (0, 2));
        nav.focus_next_option();
        assert_eq!(nav.focused_value.as_deref(), Some("c"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (1, 3));
    }

    #[test]
    fn page_navigation_moves_by_visible_count() {
        let mut nav = state(2, &["a", "b", "c", "d", "e"], None);
        nav.focus_next_page();
        assert_eq!(nav.focused_value.as_deref(), Some("c"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (1, 3));
        nav.focus_previous_page();
        assert_eq!(nav.focused_value.as_deref(), Some("a"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (0, 2));
    }

    #[test]
    fn set_focus_scrolls_minimally_to_edges() {
        let mut nav = state(2, &["a", "b", "c", "d"], None);
        nav.set_focus("d");
        assert_eq!(nav.focused_value.as_deref(), Some("d"));
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (2, 4));
        nav.set_focus("a");
        assert_eq!((nav.visible_from_index, nav.visible_to_index), (0, 2));
    }

    #[test]
    fn validated_focus_falls_back_to_first_when_missing() {
        let mut nav = state(3, &["a", "b"], Some("b"));
        nav.focused_value = Some("gone".to_string());
        assert_eq!(nav.validated_focused_value().as_deref(), Some("a"));
    }
}
