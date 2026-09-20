//! Maps to: CC
//! `components/permissions/AskUserQuestionPermissionRequest/SubmitQuestionsView.tsx`.

use super::question_navigation_bar::QuestionNavigationBar;
use super::use_multiple_choice_state::{AnswerValue, Question, all_questions_answered};
use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use crate::components::design_system::divider::Divider;
use crate::components::permissions::permission_request_title::PermissionRequestTitle;
use crate::components::permissions::permission_rule_explanation::{
    PermissionRuleExplanation, PermissionRuleToolType,
};
use crate::constants::figures::figures;
use crate::types::permissions::PermissionMode;
use crate::utils::permissions::permission_result::PermissionDecisionReason;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitQuestionsResponse {
    Submit,
    Cancel,
}

#[derive(Default, Props)]
pub struct SubmitQuestionsViewProps {
    pub questions: Vec<Question>,
    pub current_question_index: usize,
    pub answers: BTreeMap<String, AnswerValue>,
    /// Maps to: CC `SubmitQuestionsView.tsx:81-84` forwarding
    /// `permissionResult` (i.e. its `decisionReason`) to
    /// `PermissionRuleExplanation`.
    pub decision_reason: Option<PermissionDecisionReason>,
    pub permission_mode: PermissionMode,
    pub min_content_height: Option<u32>,
    pub focused_index: usize,
    pub on_final_response: Handler<SubmitQuestionsResponse>,
}

fn submit_options() -> Vec<SelectOptionData> {
    vec![
        SelectOptionData {
            label: "Submit answers".to_string(),
            value: "submit".to_string(),
            ..SelectOptionData::default()
        },
        SelectOptionData {
            label: "Cancel".to_string(),
            value: "cancel".to_string(),
            ..SelectOptionData::default()
        },
    ]
}

/// Maps to: CC `SubmitQuestionsView`.
#[component]
pub fn SubmitQuestionsView(
    props: &SubmitQuestionsViewProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let all_answered = all_questions_answered(&props.questions, &props.answers);
    let options = submit_options();

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            Divider(color: Some(theme.inactive))
            View(flex_direction: FlexDirection::Column, border_top: true, border_color: theme.inactive) {
                QuestionNavigationBar(
                    questions: props.questions.clone(),
                    current_question_index: props.current_question_index,
                    answers: props.answers.clone(),
                )
                PermissionRequestTitle(title: "Review your answers".to_string(), color: Some(theme.text))
                View(flex_direction: FlexDirection::Column, margin_top: 1u32, min_height: props.min_content_height.unwrap_or(0)) {
                    #(if all_answered { None } else { Some(element! {
                        View(margin_bottom: 1u32) {
                            Text(content: format!("{} You have not answered all questions", figures().warning), color: theme.warning, wrap: TextWrap::Wrap)
                        }
                    })})
                    #(if props.answers.is_empty() { None } else { Some(element! {
                        View(flex_direction: FlexDirection::Column, margin_bottom: 1u32) {
                            #(props.questions.iter().filter(|question| props.answers.contains_key(&question.question)).map(|question| {
                                let answer = props.answers.get(&question.question).cloned().unwrap_or_default();
                                element! {
                                    View(flex_direction: FlexDirection::Column, margin_left: 1u32) {
                                        Text(content: format!("{} {}", figures().bullet, question.question), wrap: TextWrap::Wrap)
                                        View(margin_left: 2u32) {
                                            Text(content: format!("{} {answer}", figures().arrow_right), color: theme.success, wrap: TextWrap::Wrap)
                                        }
                                    }
                                }
                            }))
                        }
                    })})
                    PermissionRuleExplanation(
                        decision_reason: props.decision_reason.clone(),
                        tool_type: PermissionRuleToolType::Tool,
                        permission_mode: props.permission_mode,
                    )
                    Text(content: "Ready to submit your answers?".to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
                    View(margin_top: 1u32) {
                        Select(
                            options: options,
                            focused_index: props.focused_index.min(1),
                            visible_option_count: 2usize,
                            layout: SelectLayout::Compact,
                            hide_indexes: true,
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn question(text: &str) -> Question {
        Question {
            question: text.to_string(),
            header: "Q".to_string(),
            ..Question::default()
        }
    }

    #[test]
    fn submit_questions_view_renders_review_warning_and_answers() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                SubmitQuestionsView(
                    questions: vec![question("Proceed?"), question("Style?")],
                    current_question_index: 2usize,
                    answers: BTreeMap::from([("Proceed?".to_string(), "Yes".to_string())]),
                )
            }
        }
        .render(Some(100))
        .to_string();

        assert!(text.contains("Review your answers"), "canvas=\n{text}");
        assert!(
            text.contains("You have not answered all questions"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Proceed?"), "canvas=\n{text}");
        assert!(text.contains("Yes"), "canvas=\n{text}");
        assert!(text.contains("Submit answers"), "canvas=\n{text}");
    }
}
