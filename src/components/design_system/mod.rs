//! Maps to: CC components/design-system/ — shared UI primitives.
//! CC's design-system provides reusable building blocks:
//!   Pane, Tabs, Divider, Dialog, Byline, KeyboardShortcutHint, ListItem, etc.
//! Current main-screen set intentionally excludes modal-only branches.

#![allow(dead_code)]

pub mod byline;
pub mod color;
pub mod dialog;
pub mod divider;
pub mod fuzzy_picker;
pub mod keyboard_shortcut_hint;
pub mod list_item;
pub mod loading_state;
pub mod pane;
pub mod progress_bar;
pub mod ratchet;
pub mod status_icon;
pub mod tabs;
pub mod theme_provider;
pub mod themed_box;
pub mod themed_text;
