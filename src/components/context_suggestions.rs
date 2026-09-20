//! Maps to: CC `components/ContextSuggestions.tsx`.

use crate::components::design_system::status_icon::{StatusIcon, StatusIconStatus};
use crate::constants::figures::MAIN_SYMBOLS;
use crate::utils::context_suggestions::{ContextSuggestion, SuggestionSeverity};
use crate::utils::format::format_tokens;
use iocraft::prelude::*;

/// Maps to: CC `components/ContextSuggestions.tsx` suggestion header copy.
pub fn context_suggestion_savings_text(tokens: Option<u64>) -> Option<String> {
    tokens.map(|tokens| {
        format!(
            " {} save ~{}",
            MAIN_SYMBOLS.arrow_right,
            format_tokens(tokens)
        )
    })
}

#[derive(Default, Props)]
pub struct ContextSuggestionsProps {
    pub suggestions: Vec<ContextSuggestion>,
}

/// Maps to: CC `components/ContextSuggestions.tsx#ContextSuggestions`.
#[component]
pub fn ContextSuggestions(props: &ContextSuggestionsProps) -> impl Into<AnyElement<'static>> {
    if props.suggestions.is_empty() {
        return element! { View(width: 0u32, height: 0u32) }.into_any();
    }

    let suggestions = props.suggestions.clone();

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            Text(content: "Suggestions".to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
            #(suggestions.into_iter().enumerate().map(|(index, suggestion)| {
                let savings = context_suggestion_savings_text(suggestion.savings_tokens);
                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        margin_top: if index == 0 { 0u32 } else { 1u32 },
                    ) {
                        View(flex_direction: FlexDirection::Row) {
                            StatusIcon(
                                status: match suggestion.severity {
                                    SuggestionSeverity::Info => StatusIconStatus::Info,
                                    SuggestionSeverity::Warning => StatusIconStatus::Warning,
                                },
                                with_space: true,
                            )
                            Text(content: suggestion.title, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                            #(savings.map(|text| element! { Text(content: text, dim: true, wrap: TextWrap::NoWrap) }))
                        }
                        View(margin_left: 2u32) {
                            Text(content: suggestion.detail, dim: true, wrap: TextWrap::Wrap)
                        }
                    }
                }
            }).collect::<Vec<_>>())
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn context_suggestion_savings_text_matches_official_copy() {
        assert_eq!(
            context_suggestion_savings_text(Some(12_300)),
            Some(format!(" {} save ~12.3k", MAIN_SYMBOLS.arrow_right))
        );
        assert_eq!(context_suggestion_savings_text(None), None);
    }

    #[test]
    fn context_suggestions_empty_renders_nothing() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ContextSuggestions(suggestions: Vec::<ContextSuggestion>::new())
            }
        }
        .render(Some(80))
        .to_string();
        assert_eq!(text, "");
    }

    #[test]
    fn context_suggestions_renders_title_savings_and_detail() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                ContextSuggestions(suggestions: vec![ContextSuggestion {
                    severity: SuggestionSeverity::Warning,
                    title: "Trim memory".to_string(),
                    detail: "Remove stale memory files".to_string(),
                    savings_tokens: Some(1_500),
                }])
            }
        }
        .render(Some(100))
        .to_string();

        assert!(text.contains("Suggestions"), "canvas=\n{text}");
        assert!(text.contains("Trim memory"), "canvas=\n{text}");
        assert!(text.contains("save ~1.5k"), "canvas=\n{text}");
        assert!(
            text.contains("Remove stale memory files"),
            "canvas=\n{text}"
        );
    }
}
