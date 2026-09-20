//! Maps to: CC `components/messages/SystemTextMessage.tsx`.
//! This is currently the plain text subset. It renders through
//! `MessageResponse`, matching CC's shared `⎿` response chrome for auxiliary
//! transcript output.

use crate::components::markdown::Markdown;
use crate::constants::figures::BLACK_CIRCLE;
use crate::types::message::{StopHookInfo, SystemMessageLevel};
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct SystemTextMessageProps {
    pub content: String,
    pub add_margin: bool,
    pub level: Option<SystemMessageLevel>,
}

#[component]
pub fn SystemTextMessage(
    props: &SystemTextMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    if props.content.trim().is_empty() {
        return element! { View }.into_any();
    }
    let color = match props.level.unwrap_or(SystemMessageLevel::Info) {
        SystemMessageLevel::Info => theme.inactive,
        SystemMessageLevel::Warning => theme.warning,
        SystemMessageLevel::Error => theme.error,
    };

    element! {
        View(
            flex_direction: FlexDirection::Row,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
        ) {
            Text(content: "  ⎿ ".to_string(), color: theme.inactive, wrap: TextWrap::NoWrap)
            Markdown(content: props.content.clone(), color: Some(color), dim_color: false)
        }
    }
    .into_any()
}

#[derive(Default, Props)]
pub struct StopHookSummaryMessageProps {
    pub add_margin: bool,
    pub verbose: bool,
    pub is_transcript_mode: bool,
    pub hook_label: Option<String>,
    pub hook_count: usize,
    pub hook_infos: Vec<StopHookInfo>,
    pub hook_errors: Vec<String>,
    pub prevented_continuation: bool,
    pub stop_reason: Option<String>,
    pub has_output: bool,
    pub total_duration_ms: Option<u64>,
}

#[component]
pub fn StopHookSummaryMessage(
    props: &StopHookSummaryMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let is_labeled = props.hook_label.is_some();

    // Official `StopHookSummaryMessage` only shows unlabeled stop summaries when
    // they have errors or prevented continuation. Labeled pre/post summaries are
    // always visible as child `⎿` rows.
    if !is_labeled && props.hook_errors.is_empty() && !props.prevented_continuation {
        return element! { View }.into_any();
    }

    let label = props
        .hook_label
        .clone()
        .unwrap_or_else(|| "stop".to_string());
    let noun = if props.hook_count == 1 {
        "hook"
    } else {
        "hooks"
    };
    let summary = format!("Ran {} {} {}", props.hook_count, label, noun);
    let info_lines = props
        .hook_infos
        .iter()
        .map(stop_hook_info_line)
        .collect::<Vec<_>>();
    let stop_reason = props.stop_reason.clone().unwrap_or_default();
    let error_lines = props
        .hook_errors
        .iter()
        .map(|error| {
            format!(
                "{} hook error: {}",
                props.hook_label.as_deref().unwrap_or("Stop"),
                error
            )
        })
        .collect::<Vec<_>>();

    if is_labeled {
        return element! {
            View(flex_direction: FlexDirection::Column) {
                Text(content: format!("  ⎿  {}", summary), color: theme.inactive, wrap: TextWrap::NoWrap)
                #(if props.is_transcript_mode && !info_lines.is_empty() {
                    Some(element! {
                        View(flex_direction: FlexDirection::Column) {
                            #(info_lines.iter().map(|line| element! {
                                Text(content: format!("     ⎿ {}", line), color: theme.inactive, wrap: TextWrap::NoWrap)
                            }))
                        }
                    })
                } else {
                    None
                })
            }
        }
        .into_any();
    }

    element! {
        View(
            flex_direction: FlexDirection::Row,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
        ) {
            View(width: 2u32, flex_shrink: 0.0f32) {
                Text(content: BLACK_CIRCLE.to_string(), color: theme.text, wrap: TextWrap::NoWrap)
            }
            View(flex_direction: FlexDirection::Column) {
                Text(content: summary, color: theme.text, wrap: TextWrap::NoWrap)
                #(if props.verbose && !info_lines.is_empty() {
                    Some(element! {
                        View(flex_direction: FlexDirection::Column) {
                            #(info_lines.iter().map(|line| element! {
                                Text(content: format!("⎿  {}", line), color: theme.inactive, wrap: TextWrap::NoWrap)
                            }))
                        }
                    })
                } else {
                    None
                })
                #(if props.prevented_continuation && !stop_reason.is_empty() {
                    Some(element! {
                        Text(content: format!("⎿  {}", stop_reason), color: theme.text, wrap: TextWrap::NoWrap)
                    })
                } else {
                    None
                })
                #(error_lines.iter().map(|line| element! {
                    Text(content: format!("⎿  {}", line), color: theme.text, wrap: TextWrap::NoWrap)
                }))
            }
        }
    }
    .into_any()
}

fn stop_hook_info_line(info: &StopHookInfo) -> String {
    match (info.command.as_deref(), info.prompt_text.as_deref()) {
        (Some("prompt"), Some(prompt)) => format!("prompt: {}", prompt),
        (Some(command), _) => command.to_string(),
        (None, Some(prompt)) => format!("prompt: {}", prompt),
        (None, None) => info
            .output
            .as_deref()
            .or(info.error.as_deref())
            .unwrap_or("hook")
            .to_string(),
    }
}
