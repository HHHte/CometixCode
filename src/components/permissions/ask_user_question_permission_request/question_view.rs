//! Maps to: CC
//! `components/permissions/AskUserQuestionPermissionRequest/QuestionView.tsx`.
//!
//! The official component owns question rendering and delegates preview-mode
//! questions to `PreviewQuestionView`. Rust keeps the same boundary and renders
//! the `Other` choice through the shared live input-option boundary.

use super::preview_question_view::PreviewQuestionView;
use super::question_navigation_bar::QuestionNavigationBar;
use super::use_multiple_choice_state::{
    AnswerValue, Question, QuestionState, question_has_preview,
};
use crate::components::custom_select::{
    Select, SelectInputOptionData, SelectLayout, SelectOptionData,
};
use crate::components::design_system::divider::Divider;
use crate::components::permissions::permission_request_title::PermissionRequestTitle;
use crate::components::prompt_input::input_paste::PastedContent;
use crate::constants::figures::figures;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Props)]
pub struct QuestionViewProps {
    pub question: Question,
    pub questions: Vec<Question>,
    pub current_question_index: usize,
    pub answers: BTreeMap<String, AnswerValue>,
    pub question_states: BTreeMap<String, QuestionState>,
    pub hide_submit_tab: bool,
    pub plan_file_path: Option<String>,
    pub min_content_height: Option<u32>,
    pub min_content_width: Option<usize>,
    pub focused_index: usize,
    pub multi_submit_focused: bool,
    pub footer_focused: bool,
    pub footer_index: usize,
    pub notes_focused: bool,
    pub external_editor_available: bool,
    pub editor_error: Option<String>,
    pub pasted_contents: BTreeMap<usize, PastedContent>,
    pub on_remove_image: Handler<usize>,
    pub on_image_paste: Handler<crate::utils::image_paste::ClipboardImage>,
    pub on_open_editor: Handler<String>,
    pub clipboard_image_override: Option<crate::utils::image_paste::ClipboardImage>,
}

/// Maps to: CC `textOptions` + automatic `Other` option.
pub fn question_select_options(
    question: &Question,
    state: Option<&QuestionState>,
) -> Vec<SelectOptionData> {
    let selected_values = state
        .map(|state| state.selected_value.clone())
        .unwrap_or_default();
    let mut options = question
        .options
        .iter()
        .map(|option| {
            let selected = selected_values.iter().any(|value| value == &option.label);
            SelectOptionData {
                label: if question.multi_select {
                    format!(
                        "[{}] {}",
                        if selected { figures().tick } else { " " },
                        option.label
                    )
                } else {
                    option.label.clone()
                },
                description: Some(option.description.clone())
                    .filter(|description| !description.is_empty()),
                dim_description: true,
                value: option.label.clone(),
                disabled: false,
                input: None,
            }
        })
        .collect::<Vec<_>>();
    options.push(SelectOptionData {
        label: if question.multi_select {
            let selected = selected_values.iter().any(|value| value == "__other__");
            format!("[{}] Other", if selected { figures().tick } else { " " })
        } else {
            "Other".to_string()
        },
        description: None,
        dim_description: true,
        value: "__other__".to_string(),
        disabled: false,
        input: Some(SelectInputOptionData {
            placeholder: Some(if question.multi_select {
                "Type something".to_string()
            } else {
                "Type something.".to_string()
            }),
            value: state
                .map(|state| state.text_input_value.clone())
                .unwrap_or_default(),
            show_label_with_value: true,
            label_value_separator: Some(": ".to_string()),
        }),
    });
    options
}

/// Maps to: CC `QuestionView`.
#[component]
pub fn QuestionView(props: &QuestionViewProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let state = props.question_states.get(&props.question.question);

    if question_has_preview(&props.question) {
        return element! {
            PreviewQuestionView(
                question: props.question.clone(),
                questions: props.questions.clone(),
                current_question_index: props.current_question_index,
                answers: props.answers.clone(),
                question_states: props.question_states.clone(),
                hide_submit_tab: props.hide_submit_tab,
                min_content_height: props.min_content_height,
                min_content_width: props.min_content_width,
                focused_index: props.focused_index,
                notes_focused: props.notes_focused,
                external_editor_available: props.external_editor_available,
                editor_error: props.editor_error.clone(),
            )
        }
        .into_any();
    }

    let options = question_select_options(&props.question, state);
    let focused = props.focused_index.min(options.len().saturating_sub(1));
    let other_focused =
        focused + 1 == options.len() && !props.footer_focused && !props.multi_submit_focused;

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(if props.plan_file_path.is_some() { Some(element! {
                View(flex_direction: FlexDirection::Column) {
                    Divider(color: Some(theme.inactive))
                    Text(content: format!("Planning: {}", props.plan_file_path.clone().unwrap_or_default()), color: theme.inactive, wrap: TextWrap::Wrap)
                }
            })} else { None })
            View(margin_top: 0u32) {
                Divider(color: Some(theme.inactive))
            }
            View(flex_direction: FlexDirection::Column) {
                QuestionNavigationBar(
                    questions: props.questions.clone(),
                    current_question_index: props.current_question_index,
                    answers: props.answers.clone(),
                    hide_submit_tab: props.hide_submit_tab,
                )
                PermissionRequestTitle(title: props.question.question.clone(), color: Some(theme.text))
                View(flex_direction: FlexDirection::Column, min_height: props.min_content_height.unwrap_or(0)) {
                    View(margin_top: 1u32) {
                        Select(
                            options: options.clone(),
                            focused_index: focused,
                            visible_option_count: options.len().max(1),
                            selected_value: state.and_then(|state| state.selected_value.first().cloned()),
                            layout: SelectLayout::CompactVertical,
                            hide_indexes: false,
                            is_disabled: props.footer_focused || props.multi_submit_focused,
                            pasted_contents: props.pasted_contents.clone(),
                            on_remove_image: props.on_remove_image.clone(),
                            on_image_paste: props.on_image_paste.clone(),
                            on_open_editor: props.on_open_editor.clone(),
                            clipboard_image_override: props.clipboard_image_override.clone(),
                        )
                    }
                    #(if props.question.multi_select {
                        Some(element! {
                            View(flex_direction: FlexDirection::Row, column_gap: 1u32) {
                                Text(content: if props.multi_submit_focused { figures().pointer.to_string() } else { " ".to_string() }, color: props.multi_submit_focused.then_some(theme.suggestion), wrap: TextWrap::NoWrap)
                                View(margin_left: 3u32) {
                                    Text(
                                        content: if props.current_question_index + 1 == props.questions.len() { "Submit".to_string() } else { "Next".to_string() },
                                        color: props.multi_submit_focused.then_some(theme.suggestion),
                                        weight: Weight::Bold,
                                        wrap: TextWrap::NoWrap,
                                    )
                                }
                            }
                        })
                    } else { None })
                    View(flex_direction: FlexDirection::Column) {
                        Divider(color: Some(theme.inactive))
                        View(flex_direction: FlexDirection::Row, column_gap: 1u32) {
                            Text(content: if props.footer_focused && props.footer_index == 0 { figures().pointer.to_string() } else { " ".to_string() }, color: if props.footer_focused && props.footer_index == 0 { Some(theme.suggestion) } else { None }, wrap: TextWrap::NoWrap)
                            Text(content: format!("{}. Chat about this", options.len() + 1), color: if props.footer_focused && props.footer_index == 0 { Some(theme.suggestion) } else { None }, wrap: TextWrap::NoWrap)
                        }
                        #(if props.plan_file_path.is_some() { Some(element! {
                            View(flex_direction: FlexDirection::Row, column_gap: 1u32) {
                                Text(content: if props.footer_focused && props.footer_index == 1 { figures().pointer.to_string() } else { " ".to_string() }, color: if props.footer_focused && props.footer_index == 1 { Some(theme.suggestion) } else { None }, wrap: TextWrap::NoWrap)
                                Text(content: format!("{}. Skip interview and plan immediately", options.len() + 2), color: if props.footer_focused && props.footer_index == 1 { Some(theme.suggestion) } else { None }, wrap: TextWrap::NoWrap)
                            }
                        })} else { None })
                    }
                    View(margin_top: 1u32) {
                        Text(
                            content: {
                                let navigation = if props.questions.len() == 1 {
                                    format!("{}/{} to navigate", figures().arrow_up, figures().arrow_down)
                                } else {
                                    "Tab/Arrow keys to navigate".to_string()
                                };
                                format!(
                                    "Enter to select · {navigation}{} · Esc to cancel",
                                    if other_focused && props.external_editor_available {
                                        " · ctrl+g to edit"
                                    } else {
                                        ""
                                    },
                                )
                            },
                            color: theme.inactive,
                            dim: true,
                            wrap: TextWrap::Wrap,
                        )
                    }
                    #(props.editor_error.clone().map(|error| element! {
                        Text(content: error, color: theme.error, wrap: TextWrap::Wrap)
                    }))
                }
            }
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn question() -> Question {
        Question {
            question: "Which library?".to_string(),
            header: "Library".to_string(),
            options: vec![
                super::super::use_multiple_choice_state::QuestionOption {
                    label: "Serde".to_string(),
                    description: "Use serde".to_string(),
                    preview: None,
                },
                super::super::use_multiple_choice_state::QuestionOption {
                    label: "Manual".to_string(),
                    description: "Write parser".to_string(),
                    preview: None,
                },
            ],
            multi_select: false,
        }
    }

    #[test]
    fn question_select_options_adds_live_other_input_like_official() {
        let state = QuestionState {
            selected_value: vec!["__other__".to_string()],
            text_input_value: "custom value".to_string(),
        };
        let options = question_select_options(&question(), Some(&state));
        assert_eq!(options.len(), 3);
        assert_eq!(options[2].label, "Other");
        let input = options[2].input.as_ref().expect("Other input");
        assert_eq!(input.value, "custom value");
        assert_eq!(input.placeholder.as_deref(), Some("Type something."));
    }

    #[test]
    fn question_view_renders_options_footer_and_help() {
        let q = question();
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                QuestionView(
                    question: q.clone(),
                    questions: vec![q],
                    answers: BTreeMap::new(),
                    focused_index: 0usize,
                )
            }
        }
        .render(Some(100))
        .to_string();

        assert!(text.contains("Which library?"), "canvas=\n{text}");
        assert!(text.contains("Serde"), "canvas=\n{text}");
        assert!(text.contains("Other"), "canvas=\n{text}");
        assert!(text.contains("Chat about this"), "canvas=\n{text}");
    }
}
