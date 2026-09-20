//! Maps to: CC `components/CustomSelect/` — the full family:
//! `select.tsx` (Select), `select-option.tsx` (SelectOption),
//! `select-input-option.tsx` (SelectInputOption), `SelectMulti.tsx`,
//! `option-map.ts`, `use-select-navigation.ts`, `use-select-state.ts`,
//! `use-select-input.ts`.
//!
//! `Select` itself renders from explicit props; the state machine lives in
//! `use_select_state`/`use_select_input` for consumers to own (existing
//! callers like LogSelector still drive it with their own state — their
//! migration onto the hooks is tracked separately).

pub mod option_map;
pub mod select;
pub mod select_input_option;
pub mod select_multi;
pub mod select_option;
pub mod use_multi_select_state;
pub mod use_select_input;
pub mod use_select_navigation;
pub mod use_select_state;

pub use option_map::{OptionMap, OptionMapItem};
pub use select::{Select, SelectInputOptionData, SelectLayout, SelectOptionData};
pub use select_input_option::SelectInputOption;
pub use select_multi::SelectMulti;
pub use select_option::SelectOption;
pub use use_select_input::{
    DisableSelection, SelectInputEvents, SelectInputOptionMeta, UseSelectInputOptions,
    use_select_input,
};
pub use use_select_navigation::{
    SelectNavigation, SelectNavigationState, UseSelectNavigationProps, use_select_navigation,
};
pub use use_select_state::{SelectState, UseSelectStateProps, use_select_state};
