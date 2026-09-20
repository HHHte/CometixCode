//! Maps to: CC `components/CustomSelect/select-option.tsx` — a thin wrapper
//! over the design-system ListItem with `styled=false` (the Select layouts
//! color their own text). Note CC passes no `disabled` through here: a
//! disabled option dims its label but keeps the normal indicator behavior.

use crate::components::design_system::list_item::ListItem;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct SelectOptionProps {
    pub is_focused: bool,
    pub is_selected: bool,
    /// Maps to: CC `description` (select-option.tsx:23) — forwarded to
    /// ListItem, which renders it below the row at paddingLeft 2.
    pub description: Option<String>,
    pub show_scroll_down: bool,
    pub show_scroll_up: bool,
    /// Maps to: CC `declareCursor` — set false when a child (e.g. a text
    /// input) declares its own cursor.
    pub declare_cursor: Option<bool>,
    pub children: Vec<AnyElement<'static>>,
}

/// Maps to: CC `SelectOption`.
#[component]
pub fn SelectOption(props: &mut SelectOptionProps) -> impl Into<AnyElement<'static>> {
    element! {
        ListItem(
            is_focused: props.is_focused,
            is_selected: props.is_selected,
            description: props.description.take(),
            show_scroll_down: props.show_scroll_down,
            show_scroll_up: props.show_scroll_up,
            styled: Some(false),
            declare_cursor: props.declare_cursor,
        ) {
            #(props.children.drain(..))
        }
    }
}
