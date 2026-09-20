//! Maps to: CC `components/CustomSelect/use-select-state.ts`.
//!
//! Combines the navigation state with the committed `value` (CC useState)
//! and `selectFocusedOption`. CC also threads onChange/onCancel through this
//! state for use-select-input; in Rust those callbacks stay with the
//! consumer — `use_select_input` surfaces accept/cancel as take-able events
//! the component body maps onto its `HandlerMut` props.

use super::use_select_navigation::{
    SelectNavigation, UseSelectNavigationProps, use_select_navigation,
};
use iocraft::prelude::*;

pub struct UseSelectStateProps {
    /// Maps to: CC `visibleOptionCount` (None = show all options).
    pub visible_option_count: Option<usize>,
    /// Option values in display order.
    pub values: Vec<String>,
    /// Maps to: CC `defaultValue` — the initially committed value.
    pub default_value: Option<String>,
    /// Maps to: CC `focusValue` (defaultFocusValue is passed here by
    /// Select, matching CC's `focusValue: defaultFocusValue`).
    pub focus_value: Option<String>,
}

/// Maps to: CC `SelectState<T>` — navigation plus the committed value.
#[derive(Clone, Copy)]
pub struct SelectState {
    pub navigation: SelectNavigation,
    pub value: State<Option<String>>,
}

impl SelectState {
    /// Maps to: CC `selectFocusedOption`.
    pub fn select_focused_option(&self) {
        let mut value = self.value;
        value.set(self.navigation.focused_value());
    }

    pub fn focused_value(&self) -> Option<String> {
        self.navigation.focused_value()
    }

    pub fn committed_value(&self) -> Option<String> {
        self.value.read().clone()
    }
}

/// Maps to: CC `useSelectState`.
pub fn use_select_state(hooks: &mut Hooks, props: UseSelectStateProps) -> SelectState {
    let value = hooks.use_state({
        let default_value = props.default_value.clone();
        move || default_value
    });
    let navigation = use_select_navigation(
        hooks,
        UseSelectNavigationProps {
            visible_option_count: props.visible_option_count,
            values: props.values,
            // CC useSelectState passes initialFocusValue: undefined and
            // focusValue: focusValue.
            initial_focus_value: None,
            focus_value: props.focus_value,
        },
    );
    SelectState { navigation, value }
}
