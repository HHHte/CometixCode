//! Maps to: CC `components/SkillImprovementSurvey.tsx`.

use crate::constants::figures::{BLACK_CIRCLE, BULLET_OPERATOR};
use crate::utils::string_utils::normalize_full_width_digits;
use iocraft::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillUpdate {
    pub change: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillImprovementSurveyResponse {
    Good,
    Dismissed,
}

impl SkillImprovementSurveyResponse {
    pub fn as_official_str(self) -> &'static str {
        match self {
            Self::Good => "good",
            Self::Dismissed => "dismissed",
        }
    }
}

#[derive(Default, Props)]
pub struct SkillImprovementSurveyProps {
    pub is_open: bool,
    pub skill_name: String,
    pub updates: Vec<SkillUpdate>,
    pub input_value: String,
}

/// Maps to: CC `FeedbackSurveyView.tsx#isValidResponseInput`.
pub fn is_valid_feedback_response_input(input: &str) -> bool {
    matches!(input, "0" | "1" | "2" | "3")
}

/// Maps to: CC `SkillImprovementSurvey.tsx#isValidInput`.
pub fn is_valid_skill_improvement_input(input: &str) -> bool {
    matches!(input, "0" | "1")
}

/// Maps to: CC `SkillImprovementSurvey.tsx` outer visibility guards.
pub fn skill_improvement_survey_should_render(is_open: bool, input_value: &str) -> bool {
    is_open && (input_value.is_empty() || is_valid_feedback_response_input(input_value))
}

/// Maps to: CC `SkillImprovementSurveyView` input effect: 1 => good/apply,
/// 0 => dismissed.
pub fn skill_improvement_survey_response_from_input(
    initial_input_value: &str,
    input_value: &str,
) -> Option<(String, SkillImprovementSurveyResponse)> {
    if input_value == initial_input_value || input_value.is_empty() {
        return None;
    }
    let last_char = input_value.chars().last()?.to_string();
    let normalized = normalize_full_width_digits(&last_char);
    if !is_valid_skill_improvement_input(&normalized) {
        return None;
    }
    let mut next_input = input_value.to_string();
    next_input.pop();
    let response = if normalized == "1" {
        SkillImprovementSurveyResponse::Good
    } else {
        SkillImprovementSurveyResponse::Dismissed
    };
    Some((next_input, response))
}

/// Maps to: CC `components/SkillImprovementSurvey.tsx#SkillImprovementSurvey`.
#[component]
pub fn SkillImprovementSurvey(
    props: &SkillImprovementSurveyProps,
) -> impl Into<AnyElement<'static>> {
    if !skill_improvement_survey_should_render(props.is_open, &props.input_value) {
        return element! { View(width: 0u32, height: 0u32) }.into_any();
    }

    let updates = props.updates.clone();

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            View(flex_direction: FlexDirection::Row) {
                Text(content: format!("{BLACK_CIRCLE} "), color: Color::Cyan, wrap: TextWrap::NoWrap)
                Text(content: format!("Skill improvement suggested for \"{}\"", props.skill_name), weight: Weight::Bold, wrap: TextWrap::NoWrap)
            }
            View(flex_direction: FlexDirection::Column, margin_left: 2u32) {
                #(updates.into_iter().map(|update| element! {
                    Text(content: format!("{BULLET_OPERATOR} {}", update.change), dim: true, wrap: TextWrap::Wrap)
                }).collect::<Vec<_>>())
            }
            View(flex_direction: FlexDirection::Row, margin_left: 2u32, margin_top: 1u32) {
                View(width: 12u32, flex_direction: FlexDirection::Row) {
                    Text(content: "1".to_string(), color: Color::Cyan, wrap: TextWrap::NoWrap)
                    Text(content: ": Apply".to_string(), wrap: TextWrap::NoWrap)
                }
                View(width: 14u32, flex_direction: FlexDirection::Row) {
                    Text(content: "0".to_string(), color: Color::Cyan, wrap: TextWrap::NoWrap)
                    Text(content: ": Dismiss".to_string(), wrap: TextWrap::NoWrap)
                }
            }
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_improvement_survey_visibility_matches_official_outer_gate() {
        assert!(skill_improvement_survey_should_render(true, ""));
        assert!(skill_improvement_survey_should_render(true, "1"));
        assert!(skill_improvement_survey_should_render(true, "3"));
        assert!(!skill_improvement_survey_should_render(false, "1"));
        assert!(!skill_improvement_survey_should_render(true, "hello"));
        assert!(!skill_improvement_survey_should_render(true, "12"));
    }

    #[test]
    fn skill_improvement_survey_response_accepts_half_and_full_width_digits() {
        assert_eq!(
            skill_improvement_survey_response_from_input("", "1"),
            Some((String::new(), SkillImprovementSurveyResponse::Good))
        );
        assert_eq!(
            skill_improvement_survey_response_from_input("", "０"),
            Some((String::new(), SkillImprovementSurveyResponse::Dismissed))
        );
        assert_eq!(skill_improvement_survey_response_from_input("", "2"), None);
        assert_eq!(skill_improvement_survey_response_from_input("1", "1"), None);
    }

    #[test]
    fn skill_improvement_survey_renders_official_copy_updates_and_options() {
        let text = element! {
            SkillImprovementSurvey(
                is_open: true,
                skill_name: "rust-review".to_string(),
                input_value: String::new(),
                updates: vec![
                    SkillUpdate { change: "Add a checklist".to_string() },
                    SkillUpdate { change: "Prefer cargo test".to_string() },
                ],
            )
        }
        .render(Some(120))
        .to_string();

        assert!(
            text.contains("Skill improvement suggested for \"rust-review\""),
            "canvas=\n{text}"
        );
        assert!(text.contains("Add a checklist"), "canvas=\n{text}");
        assert!(text.contains("Prefer cargo test"), "canvas=\n{text}");
        assert!(text.contains("1: Apply"), "canvas=\n{text}");
        assert!(text.contains("0: Dismiss"), "canvas=\n{text}");
    }

    #[test]
    fn skill_improvement_survey_closed_renders_empty() {
        let text = element! {
            SkillImprovementSurvey(is_open: false, skill_name: "skill".to_string())
        }
        .render(Some(80))
        .to_string();
        assert_eq!(text, "");
    }
}
