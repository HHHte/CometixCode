//! Maps to: CC `components/CustomSelect/use-multi-select-state.ts`.
//! Retained hook state itself lives in the iocraft `SelectMulti` component;
//! this module owns the official pure initialization/update semantics used by
//! that retained owner.

use super::select::SelectOptionData;
use std::collections::BTreeMap;

/// Maps to `initialFocusLast` / `focusValue` initialization through
/// `useSelectNavigation`.
pub fn initial_focus_index(
    options: &[SelectOptionData],
    focus_value: Option<&str>,
    initial_focus_last: bool,
) -> usize {
    focus_value
        .and_then(|value| options.iter().position(|option| option.value == value))
        .unwrap_or_else(|| {
            if initial_focus_last {
                options.len().saturating_sub(1)
            } else {
                0
            }
        })
}

/// Maps to the hook's lazy `inputValues` initializer.
pub fn initial_input_values(options: &[SelectOptionData]) -> BTreeMap<String, String> {
    options
        .iter()
        .filter_map(|option| {
            option
                .input
                .as_ref()
                .map(|input| (option.value.clone(), input.value.clone()))
        })
        .collect()
}

/// Maps to `updateInputValue`: non-empty input selects the option, empty input
/// removes it. Returns the next selection without mutating caller state.
pub fn update_input_value_selection(
    selected: &[String],
    option_value: &str,
    input_value: &str,
) -> Vec<String> {
    let mut next = selected.to_vec();
    if input_value.is_empty() {
        next.retain(|value| value != option_value);
    } else if !next.iter().any(|value| value == option_value) {
        next.push(option_value.to_string());
    }
    next
}

pub use super::select_multi::{SelectMulti, SelectMultiProps};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::custom_select::select::{SelectInputOptionData, SelectOptionData};

    fn option(value: &str) -> SelectOptionData {
        SelectOptionData {
            value: value.to_string(),
            label: value.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn initial_focus_matches_explicit_and_last_rules() {
        let options = vec![option("a"), option("b")];
        assert_eq!(initial_focus_index(&options, Some("b"), false), 1);
        assert_eq!(initial_focus_index(&options, None, true), 1);
        assert_eq!(initial_focus_index(&options, None, false), 0);
    }

    #[test]
    fn input_values_and_selection_match_hook_semantics() {
        let options = vec![SelectOptionData {
            value: "extra".to_string(),
            input: Some(SelectInputOptionData {
                value: "seed".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }];
        assert_eq!(
            initial_input_values(&options).get("extra"),
            Some(&"seed".to_string())
        );
        assert_eq!(
            update_input_value_selection(&[], "extra", "x"),
            vec!["extra".to_string()]
        );
        assert!(update_input_value_selection(&["extra".to_string()], "extra", "").is_empty());
    }
}
