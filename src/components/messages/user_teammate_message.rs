//! Maps to: CC `components/messages/UserTeammateMessage.tsx`.

use crate::components::markdown::Markdown;
use crate::components::message_response::MessageResponse;
use crate::constants::figures::figures;
use crate::constants::xml::TEAMMATE_MESSAGE_TAG;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedTeammateMessage {
    teammate_id: String,
    content: String,
    color: Option<String>,
    summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StructuredTeammateMessage {
    PlanApprovalRequest {
        from: String,
        plan_file_path: String,
        plan_content: String,
    },
    PlanApprovalResponse {
        approved: bool,
        feedback: Option<String>,
    },
    ShutdownRequest {
        from: String,
        reason: Option<String>,
    },
    ShutdownRejected {
        from: String,
        reason: String,
    },
    ShutdownApproved,
    TaskAssignment {
        task_id: String,
        subject: String,
        description: Option<String>,
        assigned_by: String,
    },
    TaskCompleted {
        task_id: String,
        task_subject: Option<String>,
    },
    IdleNotification,
    TeammateTerminated,
}

#[derive(Default, Props)]
pub struct UserTeammateMessageProps {
    /// Raw text block containing one or more `<teammate-message>` records, or a
    /// plain teammate message when rendered through the typed Rust enum.
    pub content: String,
    /// Fallback sender for typed Rust messages that are already split at the
    /// transcript boundary.
    pub sender: String,
    pub add_margin: bool,
    pub is_transcript_mode: bool,
}

#[component]
pub fn UserTeammateMessage(
    props: &UserTeammateMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let messages = parse_or_fallback_teammate_messages(&props.content, &props.sender)
        .into_iter()
        .filter(|message| !should_hide_message(&message.content))
        .collect::<Vec<_>>();

    if messages.is_empty() {
        return element! { View }.into_any();
    }

    element! {
        View(
            flex_direction: FlexDirection::Column,
            margin_top: if props.add_margin { 1u32 } else { 0u32 },
            width: 100pct,
        ) {
            #(messages.into_iter().map(|message| {
                let display_name = display_name(&message.teammate_id);
                let color = teammate_color(message.color.as_deref(), &theme);
                render_teammate_message(
                    display_name,
                    color,
                    message.content,
                    message.summary,
                    props.is_transcript_mode,
                    &theme,
                )
            }))
        }
    }
    .into_any()
}

fn render_teammate_message(
    display_name: String,
    color: Option<Color>,
    content: String,
    summary: Option<String>,
    is_transcript_mode: bool,
    theme: &Theme,
) -> AnyElement<'static> {
    match parse_structured_teammate_message(&content) {
        Some(StructuredTeammateMessage::PlanApprovalRequest {
            from,
            plan_file_path,
            plan_content,
        }) => element! {
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1u32,
                border_style: BorderStyle::Round,
                border_color: theme.plan_mode,
                padding_left: 1u32,
                padding_right: 1u32,
            ) {
                Text(content: format!("Plan Approval Request from {from}"), color: theme.plan_mode, weight: Weight::Bold)
                View(
                    flex_direction: FlexDirection::Column,
                    border_style: BorderStyle::Dashed,
                    border_color: theme.subtle,
                    border_left: false,
                    border_right: false,
                    padding_left: 1u32,
                    padding_right: 1u32,
                ) {
                    Markdown(content: plan_content)
                }
                Text(content: format!("Plan file: {plan_file_path}"), color: theme.inactive)
            }
        }
        .into_any(),
        Some(StructuredTeammateMessage::PlanApprovalResponse { approved, feedback }) => {
            let (label, color, border_color) = if approved {
                (
                    format!("✓ Plan Approved by {display_name}"),
                    theme.success,
                    theme.success,
                )
            } else {
                (
                    format!("✗ Plan Rejected by {display_name}"),
                    theme.error,
                    theme.error,
                )
            };
            element! {
                View(
                    flex_direction: FlexDirection::Column,
                    margin_top: 1u32,
                    border_style: BorderStyle::Round,
                    border_color: border_color,
                    padding_left: 1u32,
                    padding_right: 1u32,
                ) {
                    Text(content: label, color: color, weight: Weight::Bold)
                    #(if approved {
                        Some(element! {
                            Text(content: "You can now proceed with implementation. Your plan mode restrictions have been lifted.".to_string())
                        }.into_any())
                    } else {
                        feedback
                            .filter(|value| !value.trim().is_empty())
                            .map(|value| element! {
                                View(
                                    border_style: BorderStyle::Dashed,
                                    border_color: theme.subtle,
                                    border_left: false,
                                    border_right: false,
                                    padding_left: 1u32,
                                    padding_right: 1u32,
                                ) {
                                    Text(content: format!("Feedback: {value}"), color: theme.text)
                                }
                            }.into_any())
                    })
                    #(if approved { None } else {
                        Some(element! {
                            Text(content: "Please revise your plan based on the feedback and call ExitPlanMode again.".to_string(), color: theme.inactive)
                        })
                    })
                }
            }
            .into_any()
        }
        Some(StructuredTeammateMessage::ShutdownRequest { from, reason }) => element! {
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1u32,
                border_style: BorderStyle::Round,
                border_color: theme.warning,
                padding_left: 1u32,
                padding_right: 1u32,
            ) {
                Text(content: format!("Shutdown request from {from}"), color: theme.warning, weight: Weight::Bold)
                #(reason.filter(|value| !value.trim().is_empty()).map(|value| element! {
                    Text(content: format!("Reason: {value}"), color: theme.text)
                }))
            }
        }
        .into_any(),
        Some(StructuredTeammateMessage::ShutdownRejected { from, reason }) => element! {
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1u32,
                border_style: BorderStyle::Round,
                border_color: theme.subtle,
                padding_left: 1u32,
                padding_right: 1u32,
            ) {
                Text(content: format!("Shutdown rejected by {from}"), color: theme.subtle, weight: Weight::Bold)
                View(
                    border_style: BorderStyle::Dashed,
                    border_color: theme.subtle,
                    border_left: false,
                    border_right: false,
                    padding_left: 1u32,
                    padding_right: 1u32,
                ) {
                    Text(content: format!("Reason: {reason}"), color: theme.text)
                }
                Text(content: "Teammate is continuing to work. You may request shutdown again later.".to_string(), color: theme.inactive)
            }
        }
        .into_any(),
        Some(StructuredTeammateMessage::TaskAssignment {
            task_id,
            subject,
            description,
            assigned_by,
        }) => element! {
            View(
                flex_direction: FlexDirection::Column,
                margin_top: 1u32,
                border_style: BorderStyle::Round,
                border_color: theme.agent_cyan,
                padding_left: 1u32,
                padding_right: 1u32,
            ) {
                Text(content: format!("Task #{task_id} assigned by {assigned_by}"), color: theme.agent_cyan, weight: Weight::Bold)
                Text(content: subject, weight: Weight::Bold)
                #(description.filter(|value| !value.trim().is_empty()).map(|value| element! {
                    Text(content: value, color: theme.inactive)
                }))
            }
        }
        .into_any(),
        Some(StructuredTeammateMessage::TaskCompleted {
            task_id,
            task_subject,
        }) => {
            let suffix = task_subject
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!(" ({value})"))
                .unwrap_or_default();
            element! {
                View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
                    Text(content: format!("@{display_name}{}", figures().pointer), color: color, wrap: TextWrap::NoWrap)
                    MessageResponse(content: format!("✓ Completed task #{task_id}{suffix}"), color: Some(theme.success))
                }
            }
            .into_any()
        }
        Some(
            StructuredTeammateMessage::ShutdownApproved
            | StructuredTeammateMessage::IdleNotification
            | StructuredTeammateMessage::TeammateTerminated,
        ) => element! { View }.into_any(),
        None => element! {
            View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
                View(flex_direction: FlexDirection::Row) {
                    Text(content: format!("@{display_name}{}", figures().pointer), color: color, wrap: TextWrap::NoWrap)
                    #(summary.filter(|value| !value.trim().is_empty()).map(|value| element! {
                        Text(content: format!(" {value}"), color: theme.text)
                    }))
                }
                #(if is_transcript_mode {
                    Some(element! {
                        View(padding_left: 2u32) {
                            Ansi(content: content)
                        }
                    })
                } else { None })
            }
        }
        .into_any(),
    }
}

fn parse_or_fallback_teammate_messages(
    text: &str,
    fallback_sender: &str,
) -> Vec<ParsedTeammateMessage> {
    let parsed = parse_teammate_messages(text);
    if !parsed.is_empty() || text.trim().is_empty() {
        return parsed;
    }

    vec![ParsedTeammateMessage {
        teammate_id: if fallback_sender.trim().is_empty() {
            "teammate".to_string()
        } else {
            fallback_sender.to_string()
        },
        content: text.trim().to_string(),
        color: None,
        summary: None,
    }]
}

fn parse_teammate_messages(text: &str) -> Vec<ParsedTeammateMessage> {
    let mut messages = Vec::new();
    let mut cursor = 0usize;
    let open_prefix = format!("<{TEAMMATE_MESSAGE_TAG}");
    let close_tag = format!("</{TEAMMATE_MESSAGE_TAG}>");

    while let Some(open_rel) = text[cursor..].find(&open_prefix) {
        let open_start = cursor + open_rel;
        let Some(open_end_rel) = text[open_start..].find('>') else {
            break;
        };
        let open_end = open_start + open_end_rel;
        let header = &text[open_start..=open_end];
        let content_start = open_end + 1;
        let Some(close_rel) = text[content_start..].find(&close_tag) else {
            break;
        };
        let content_end = content_start + close_rel;
        cursor = content_end + close_tag.len();

        let Some(teammate_id) = extract_attr(header, "teammate_id") else {
            continue;
        };
        let content = text[content_start..content_end].trim().to_string();
        if content.is_empty() {
            continue;
        }

        messages.push(ParsedTeammateMessage {
            teammate_id,
            color: extract_attr(header, "color"),
            summary: extract_attr(header, "summary"),
            content,
        });
    }

    messages
}

fn extract_attr(text: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let start = text.find(&needle)? + needle.len();
    let end = text[start..].find('"')? + start;
    Some(text[start..end].to_string())
}

fn display_name(teammate_id: &str) -> String {
    if teammate_id == "leader" {
        "leader".to_string()
    } else {
        teammate_id.to_string()
    }
}

fn teammate_color(color: Option<&str>, theme: &Theme) -> Option<Color> {
    match color.unwrap_or_default().to_ascii_lowercase().as_str() {
        "red" => Some(theme.agent_red),
        "blue" => Some(theme.agent_blue),
        "green" => Some(theme.agent_green),
        "yellow" => Some(theme.agent_yellow),
        "purple" => Some(theme.agent_purple),
        "orange" => Some(theme.agent_orange),
        "pink" => Some(theme.agent_pink),
        "cyan" => Some(theme.agent_cyan),
        _ => Some(theme.agent_cyan),
    }
}

fn should_hide_message(content: &str) -> bool {
    matches!(
        parse_structured_teammate_message(content),
        Some(
            StructuredTeammateMessage::ShutdownApproved
                | StructuredTeammateMessage::IdleNotification
                | StructuredTeammateMessage::TeammateTerminated
        )
    )
}

fn parse_structured_teammate_message(content: &str) -> Option<StructuredTeammateMessage> {
    let value = serde_json::from_str::<serde_json::Value>(content).ok()?;
    let message_type = value.get("type")?.as_str()?;
    match message_type {
        "plan_approval_request" => Some(StructuredTeammateMessage::PlanApprovalRequest {
            from: string_field(&value, "from").unwrap_or_else(|| "teammate".to_string()),
            plan_file_path: string_field(&value, "planFilePath").unwrap_or_default(),
            plan_content: string_field(&value, "planContent").unwrap_or_default(),
        }),
        "plan_approval_response" => Some(StructuredTeammateMessage::PlanApprovalResponse {
            approved: value
                .get("approved")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            feedback: string_field(&value, "feedback"),
        }),
        "shutdown_request" => Some(StructuredTeammateMessage::ShutdownRequest {
            from: string_field(&value, "from").unwrap_or_else(|| "teammate".to_string()),
            reason: string_field(&value, "reason"),
        }),
        "shutdown_approved" => Some(StructuredTeammateMessage::ShutdownApproved),
        "shutdown_rejected" => Some(StructuredTeammateMessage::ShutdownRejected {
            from: string_field(&value, "from").unwrap_or_else(|| "teammate".to_string()),
            reason: string_field(&value, "reason").unwrap_or_default(),
        }),
        "task_assignment" => Some(StructuredTeammateMessage::TaskAssignment {
            task_id: string_field(&value, "taskId").unwrap_or_default(),
            subject: string_field(&value, "subject").unwrap_or_default(),
            description: string_field(&value, "description"),
            assigned_by: string_field(&value, "assignedBy").unwrap_or_default(),
        }),
        "task_completed" => Some(StructuredTeammateMessage::TaskCompleted {
            task_id: string_field(&value, "taskId").unwrap_or_default(),
            task_subject: string_field(&value, "taskSubject"),
        }),
        "idle_notification" => Some(StructuredTeammateMessage::IdleNotification),
        "teammate_terminated" => Some(StructuredTeammateMessage::TeammateTerminated),
        _ => None,
    }
}

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_message(content: String, is_transcript_mode: bool) -> String {
        element! {
            ContextProvider(value: Context::owned(*crate::utils::theme::current())) {
                UserTeammateMessage(
                    sender: "fallback".to_string(),
                    content: content,
                    is_transcript_mode: is_transcript_mode,
                )
            }
        }
        .render(None)
        .to_string()
    }

    #[test]
    fn parses_multiple_teammate_messages_with_attributes() {
        let messages = parse_teammate_messages(
            r#"<teammate-message teammate_id="alice" color="red" summary="Brief update">one</teammate-message>
<teammate-message teammate_id="leader">two</teammate-message>"#,
        );

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].teammate_id, "alice");
        assert_eq!(messages[0].color.as_deref(), Some("red"));
        assert_eq!(messages[0].summary.as_deref(), Some("Brief update"));
        assert_eq!(messages[1].teammate_id, "leader");
        assert_eq!(messages[1].content, "two");
    }

    #[test]
    fn teammate_message_renders_multiple_headers_and_hides_content_until_transcript_mode() {
        let content = r#"<teammate-message teammate_id="alice" summary="Brief update">body one</teammate-message>
<teammate-message teammate_id="bob">body two</teammate-message>"#
            .to_string();

        let compact = render_message(content.clone(), false);
        assert!(compact.contains("@alice"));
        assert!(compact.contains("Brief update"));
        assert!(compact.contains("@bob"));
        assert!(!compact.contains("body one"));
        assert!(!compact.contains("body two"));

        let transcript = render_message(content, true);
        assert!(transcript.contains("body one"));
        assert!(transcript.contains("body two"));
    }

    #[test]
    fn teammate_message_filters_lifecycle_noise() {
        let content = r#"<teammate-message teammate_id="alice">{"type":"shutdown_approved","requestId":"r","from":"alice","timestamp":"now"}</teammate-message>
<teammate-message teammate_id="bob">{"type":"teammate_terminated","message":"done"}</teammate-message>
<teammate-message teammate_id="carol">{"type":"idle_notification"}</teammate-message>"#
            .to_string();

        let rendered = render_message(content, true);
        assert!(rendered.trim().is_empty());
    }

    #[test]
    fn teammate_message_renders_structured_task_completed() {
        let content = r#"<teammate-message teammate_id="worker">{"type":"task_completed","from":"worker","taskId":"42","taskSubject":"tests"}</teammate-message>"#
            .to_string();

        let rendered = render_message(content, false);
        assert!(rendered.contains("@worker"));
        assert!(rendered.contains("Completed task #42"));
        assert!(rendered.contains("tests"));
    }

    #[test]
    fn teammate_message_renders_plan_shutdown_and_assignment_summaries() {
        let plan_request = render_message(
            r###"<teammate-message teammate_id="leader">{"type":"plan_approval_request","requestId":"r","from":"leader","planFilePath":"/tmp/plan.md","planContent":"## Implement\n\nDo it","timestamp":"now"}</teammate-message>"###.to_string(),
            false,
        );
        assert!(plan_request.contains("Plan Approval Request from leader"));
        assert!(plan_request.contains("Implement"));
        assert!(plan_request.contains("Plan file: /tmp/plan.md"));

        let plan = render_message(
            r#"<teammate-message teammate_id="leader">{"type":"plan_approval_response","requestId":"r","approved":false,"feedback":"revise","timestamp":"now"}</teammate-message>"#.to_string(),
            false,
        );
        assert!(plan.contains("Plan Rejected by leader"));
        assert!(plan.contains("Feedback: revise"));
        assert!(plan.contains("Please revise your plan"));

        let shutdown = render_message(
            r#"<teammate-message teammate_id="leader">{"type":"shutdown_request","requestId":"r","from":"leader","reason":"done","timestamp":"now"}</teammate-message>"#.to_string(),
            false,
        );
        assert!(shutdown.contains("Shutdown request from leader"));
        assert!(shutdown.contains("Reason: done"));

        let shutdown_rejected = render_message(
            r#"<teammate-message teammate_id="leader">{"type":"shutdown_rejected","requestId":"r","from":"leader","reason":"still busy","timestamp":"now"}</teammate-message>"#.to_string(),
            false,
        );
        assert!(shutdown_rejected.contains("Shutdown rejected by leader"));
        assert!(shutdown_rejected.contains("Teammate is continuing to work"));

        let task = render_message(
            r#"<teammate-message teammate_id="lead">{"type":"task_assignment","taskId":"7","subject":"Review","description":"Check diff","assignedBy":"lead","timestamp":"now"}</teammate-message>"#.to_string(),
            false,
        );
        assert!(task.contains("Task #7 assigned by lead"));
        assert!(task.contains("Review"));
        assert!(task.contains("Check diff"));
    }
}
