//! Maps to: CC
//! `components/permissions/AskUserQuestionPermissionRequest/PreviewQuestionView.tsx`.
//!
//! Preview questions use a side-by-side option list and preview panel. Notes
//! editing and explicit external-editor handoff are owned by the parent retained
//! event loop and rendered here.

use super::preview_box::PreviewBox;
use super::question_navigation_bar::QuestionNavigationBar;
use super::use_multiple_choice_state::{AnswerValue, Question, QuestionState};
use crate::components::design_system::divider::Divider;
use crate::components::permissions::permission_request_title::PermissionRequestTitle;
use crate::constants::figures::figures;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Props)]
pub struct PreviewQuestionViewProps {
    pub question: Question,
    pub questions: Vec<Question>,
    pub current_question_index: usize,
    pub answers: BTreeMap<String, AnswerValue>,
    pub question_states: BTreeMap<String, QuestionState>,
    pub hide_submit_tab: bool,
    pub min_content_height: Option<u32>,
    pub min_content_width: Option<usize>,
    pub focused_index: usize,
    pub notes_focused: bool,
    pub external_editor_available: bool,
    pub editor_error: Option<String>,
}

/// Maps to: CC `PreviewQuestionView`.
#[component]
pub fn PreviewQuestionView(
    props: &PreviewQuestionViewProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let (columns, _) = hooks.use_terminal_size();
    let question_state = props.question_states.get(&props.question.question);
    let selected_value = question_state.and_then(|state| state.selected_value.first().cloned());
    let focused = props
        .focused_index
        .min(props.question.options.len().saturating_sub(1));
    let focused_option = props.question.options.get(focused);
    let preview = focused_option
        .and_then(|option| option.preview.clone())
        .unwrap_or_else(|| "No preview available".to_string());
    let left_panel_width = 30usize;
    let preview_max_width = (columns as usize)
        .saturating_sub(left_panel_width + 4)
        .max(20);
    let preview_max_lines = props
        .min_content_height
        .map(|height| (height as usize).saturating_sub(11).max(1));
    let notes_value = question_state
        .map(|state| state.text_input_value.clone())
        .unwrap_or_default();
    let notes_text = if props.notes_focused {
        if notes_value.is_empty() {
            "Add notes on this design…".to_string()
        } else {
            format!("{notes_value}▌")
        }
    } else if notes_value.is_empty() {
        "press n to add notes".to_string()
    } else {
        notes_value.clone()
    };

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            Divider(color: Some(theme.inactive))
            View(flex_direction: FlexDirection::Column) {
                QuestionNavigationBar(
                    questions: props.questions.clone(),
                    current_question_index: props.current_question_index,
                    answers: props.answers.clone(),
                    hide_submit_tab: props.hide_submit_tab,
                )
                PermissionRequestTitle(title: props.question.question.clone(), color: Some(theme.text))
                View(flex_direction: FlexDirection::Column, min_height: props.min_content_height.unwrap_or(0)) {
                    View(margin_top: 1u32, flex_direction: FlexDirection::Row, column_gap: 4u32) {
                        View(flex_direction: FlexDirection::Column, width: left_panel_width as u32) {
                            #(props.question.options.iter().enumerate().map(|(index, option)| {
                                let is_focused = index == focused;
                                let is_selected = selected_value.as_ref() == Some(&option.label);
                                element! {
                                    View(flex_direction: FlexDirection::Row) {
                                        Text(content: if is_focused { figures().pointer.to_string() } else { " ".to_string() }, color: if is_focused { Some(theme.suggestion) } else { None }, wrap: TextWrap::NoWrap)
                                        Text(content: format!(" {}.", index + 1), color: theme.inactive, wrap: TextWrap::NoWrap)
                                        Text(
                                            content: format!(" {}", option.label),
                                            color: if is_selected { Some(theme.success) } else if is_focused { Some(theme.suggestion) } else { None },
                                            weight: if is_focused { Weight::Bold } else { Weight::Normal },
                                            wrap: TextWrap::NoWrap,
                                        )
                                        #(if is_selected { Some(element! { Text(content: format!(" {}", figures().tick), color: theme.success, wrap: TextWrap::NoWrap) }) } else { None })
                                    }
                                }
                            }))
                        }
                        View(flex_direction: FlexDirection::Column, flex_grow: 1.0f32) {
                            PreviewBox(
                                content: preview,
                                max_lines: preview_max_lines,
                                min_width: props.min_content_width,
                                max_width: Some(preview_max_width),
                            )
                            View(margin_top: 1u32, flex_direction: FlexDirection::Row, column_gap: 1u32) {
                                Text(content: "Notes:".to_string(), color: theme.suggestion, wrap: TextWrap::NoWrap)
                                Text(content: notes_text.clone(), color: if props.notes_focused { Some(theme.suggestion) } else { Some(theme.inactive) }, dim: !props.notes_focused, wrap: TextWrap::Wrap)
                            }
                        }
                    }
                    View(flex_direction: FlexDirection::Column) {
                        Divider(color: Some(theme.inactive))
                        Text(content: format!("  {}. Chat about this", props.question.options.len() + 1), wrap: TextWrap::NoWrap)
                    }
                    View(margin_top: 1u32) {
                        Text(
                            content: format!(
                                "Enter to select · Tab/Arrow keys to navigate · n for notes{} · Esc to cancel",
                                if props.notes_focused && props.external_editor_available {
                                    " · ctrl+g to edit"
                                } else {
                                    ""
                                },
                            ),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn preview_question() -> Question {
        Question {
            question: "Which layout?".to_string(),
            header: "Layout".to_string(),
            options: vec![
                super::super::use_multiple_choice_state::QuestionOption {
                    label: "List".to_string(),
                    description: "Use rows".to_string(),
                    preview: Some("A\nB".to_string()),
                },
                super::super::use_multiple_choice_state::QuestionOption {
                    label: "Grid".to_string(),
                    description: "Use cards".to_string(),
                    preview: Some("A B".to_string()),
                },
            ],
            multi_select: false,
        }
    }

    #[test]
    fn preview_question_view_renders_side_by_side_preview() {
        let question = preview_question();
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                PreviewQuestionView(
                    question: question.clone(),
                    questions: vec![question],
                    answers: BTreeMap::new(),
                    focused_index: 0usize,
                )
            }
        }
        .render(Some(100))
        .to_string();

        assert!(text.contains("Which layout?"), "canvas=\n{text}");
        assert!(text.contains("List"), "canvas=\n{text}");
        assert!(text.contains("┌"), "canvas=\n{text}");
        assert!(text.contains("Notes:"), "canvas=\n{text}");
        assert!(text.contains("press n to add notes"), "canvas=\n{text}");
    }
}
