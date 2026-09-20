//! Maps to: CC `components/AgentProgressLine.tsx`.
//!
//! Pure renderer for one subagent progress tree row. It does not start or poll
//! agents; callers pass the already-derived progress statistics from the Agent
//! tool UI path.

use crate::utils::format::format_number;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct AgentProgressLineProps {
    pub agent_type: String,
    pub description: Option<String>,
    pub name: Option<String>,
    pub description_color: Option<Color>,
    pub task_description: Option<String>,
    pub tool_use_count: usize,
    pub tokens: Option<u64>,
    pub color: Option<Color>,
    pub is_last: bool,
    pub is_resolved: bool,
    /// CC destructures this as `_isError` (`AgentProgressLine.tsx:35`) and
    /// never reads it: a member row is default foreground with
    /// `dimColor={!isResolved}` (`:58-59`), and an errored agent looks exactly
    /// like a finished one here. Kept as a prop because CC keeps the prop.
    pub is_error: bool,
    pub is_async: bool,
    pub should_animate: bool,
    pub last_tool_info: Option<String>,
    pub hide_type: bool,
}

/// Maps to: CC `AgentProgressLine.tsx:45-53#getStatusText`.
///
/// The two fallbacks use DIFFERENT JS operators and the difference is
/// observable on an empty string: `lastToolInfo || 'Initializing…'` (`:47`) is
/// truthy, so `""` falls back; `taskDescription ?? 'Running in the background'`
/// (`:50`) is nullish, so `""` renders as an empty status line.
pub fn agent_progress_status_text(
    is_resolved: bool,
    is_backgrounded: bool,
    last_tool_info: Option<&str>,
    task_description: Option<&str>,
) -> String {
    if !is_resolved {
        return last_tool_info
            .filter(|value| !value.is_empty())
            .unwrap_or("Initializing…")
            .to_string();
    }
    if is_backgrounded {
        return task_description
            .unwrap_or("Running in the background")
            .to_string();
    }
    "Done".to_string()
}

/// Maps to: CC `toolUseCount`/`tokens` summary fragment.
pub fn agent_progress_usage_text(tool_use_count: usize, tokens: Option<u64>) -> String {
    let mut text = format!(
        "{tool_use_count} tool {}",
        if tool_use_count == 1 { "use" } else { "uses" }
    );
    if let Some(tokens) = tokens {
        text.push_str(&format!(" · {} tokens", format_number(tokens)));
    }
    text
}

/// CC `:62` `{name ?? description ?? agentType}` — NULLISH, so an empty `name`
/// or `description` still wins over the fallback behind it.
fn title_text(props: &AgentProgressLineProps) -> String {
    if props.hide_type {
        props
            .name
            .as_ref()
            .or(props.description.as_ref())
            .unwrap_or(&props.agent_type)
            .clone()
    } else {
        props.agent_type.clone()
    }
}

/// CC's JS-truthy `&&` guards (`:63`, `:74`), which an empty string fails —
/// unlike the `??` chain in [`title_text`].
fn truthy(value: &Option<String>) -> Option<&String> {
    value.as_ref().filter(|value| !value.is_empty())
}

/// Maps to: CC `components/AgentProgressLine.tsx#AgentProgressLine`.
#[component]
pub fn AgentProgressLine(
    props: &AgentProgressLineProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let tree_char = if props.is_last { "└─" } else { "├─" };
    let is_backgrounded = props.is_async && props.is_resolved;
    let status_text = agent_progress_status_text(
        props.is_resolved,
        is_backgrounded,
        props.last_tool_info.as_deref(),
        props.task_description.as_deref(),
    );
    let title = title_text(props);
    let title_color = if props.color.is_some() {
        Some(theme.inverse_text)
    } else {
        None
    };
    let description_color = if props.description_color.is_some() {
        Some(theme.inverse_text)
    } else {
        None
    };

    element! {
        View(flex_direction: FlexDirection::Column) {
            View(flex_direction: FlexDirection::Row, padding_left: 3u32) {
                Text(content: format!("{tree_char} "), dim: true, wrap: TextWrap::NoWrap)
                #(if props.hide_type {
                    Some(element! {
                        View(flex_direction: FlexDirection::Row) {
                            Text(content: title, weight: Weight::Bold, dim: !props.is_resolved, wrap: TextWrap::NoWrap)
                            // CC `:63` `{name && description && …}` — both guards truthy.
                            #(truthy(&props.name).and(truthy(&props.description)).map(|description| element! {
                                Text(
                                    content: format!(": {description}"),
                                    dim: true,
                                    wrap: TextWrap::NoWrap,
                                )
                            }))
                        }
                    }.into_any())
                } else {
                    Some(element! {
                        View(flex_direction: FlexDirection::Row) {
                            Text(
                                content: title,
                                color: title_color,
                                background_color: props.color,
                                weight: Weight::Bold,
                                dim: !props.is_resolved,
                                wrap: TextWrap::NoWrap,
                            )
                            // CC `:74` `{description && (…)}` — truthy, so an
                            // empty description renders no ` ()` suffix.
                            #(truthy(&props.description).map(|description| element! {
                                View(flex_direction: FlexDirection::Row) {
                                    Text(content: " (".to_string(), dim: !props.is_resolved, wrap: TextWrap::NoWrap)
                                    Text(
                                        content: description.clone(),
                                        color: description_color,
                                        background_color: props.description_color,
                                        dim: !props.is_resolved,
                                        wrap: TextWrap::NoWrap,
                                    )
                                    Text(content: ")".to_string(), dim: !props.is_resolved, wrap: TextWrap::NoWrap)
                                }
                            }))
                        }
                    }.into_any())
                })
                #(if is_backgrounded {
                    None
                } else {
                    Some(element! {
                        Text(
                            content: format!(" · {}", agent_progress_usage_text(props.tool_use_count, props.tokens)),
                            dim: !props.is_resolved,
                            wrap: TextWrap::NoWrap,
                        )
                    })
                })
            }
            #(if is_backgrounded {
                None
            } else {
                Some(element! {
                    View(flex_direction: FlexDirection::Row, padding_left: 3u32) {
                        Text(content: if props.is_last { "   ⎿  ".to_string() } else { "│  ⎿  ".to_string() }, dim: true, wrap: TextWrap::NoWrap)
                        Text(content: status_text, dim: true, wrap: TextWrap::NoWrap)
                    }
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn agent_progress_status_text_matches_official_branches() {
        assert_eq!(
            agent_progress_status_text(false, false, Some("Reading files"), None),
            "Reading files"
        );
        assert_eq!(
            agent_progress_status_text(false, false, None, None),
            "Initializing…"
        );
        assert_eq!(agent_progress_status_text(true, false, None, None), "Done");
        assert_eq!(
            agent_progress_status_text(true, true, None, None),
            "Running in the background"
        );
        assert_eq!(
            agent_progress_status_text(true, true, None, Some("Checking CI")),
            "Checking CI"
        );
        // `lastToolInfo || …` is truthy, `taskDescription ?? …` is nullish
        // (`:47` vs `:50`) — the empty string goes opposite ways.
        assert_eq!(
            agent_progress_status_text(false, false, Some(""), None),
            "Initializing…"
        );
        assert_eq!(agent_progress_status_text(true, true, None, Some("")), "");
    }

    /// CC `:74` guards the parenthesised description with `&&`, but `:62`
    /// selects the collapsed title with `??` — an empty description therefore
    /// titles a `hideType` row while rendering no ` ()` on a typed one.
    #[test]
    fn an_empty_description_is_truthy_falsy_for_the_parens_but_not_the_title() {
        let typed = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                AgentProgressLine(
                    agent_type: "explore".to_string(),
                    description: Some(String::new()),
                    tool_use_count: 1usize,
                    is_last: true,
                    is_resolved: true,
                )
            }
        }
        .render(Some(120))
        .to_string();
        assert!(
            typed.contains("└─ explore · 1 tool use"),
            "canvas=\n{typed}"
        );
        assert!(!typed.contains("()"), "canvas=\n{typed}");

        let collapsed = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                AgentProgressLine(
                    agent_type: "explore".to_string(),
                    description: Some(String::new()),
                    tool_use_count: 1usize,
                    is_last: true,
                    is_resolved: true,
                    hide_type: true,
                )
            }
        }
        .render(Some(120))
        .to_string();
        assert!(
            !collapsed.contains("explore"),
            "the empty description wins the ?? chain: canvas=\n{collapsed}"
        );
        assert!(
            collapsed.contains("└─  · 1 tool use"),
            "canvas=\n{collapsed}"
        );
    }

    #[test]
    fn agent_progress_usage_text_pluralizes_and_formats_tokens() {
        assert_eq!(agent_progress_usage_text(1, None), "1 tool use");
        assert_eq!(
            agent_progress_usage_text(2, Some(12_500)),
            "2 tool uses · 12.5k tokens"
        );
    }

    #[test]
    fn agent_progress_line_renders_running_tree_and_status() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                AgentProgressLine(
                    agent_type: "explore".to_string(),
                    description: Some("search repo".to_string()),
                    tool_use_count: 2usize,
                    tokens: Some(12500u64),
                    is_last: false,
                    is_resolved: false,
                    last_tool_info: Some("Reading files".to_string()),
                )
            }
        }
        .render(Some(120))
        .to_string();

        assert!(
            text.contains("├─ explore (search repo) · 2 tool uses · 12.5k tokens"),
            "canvas=\n{text}"
        );
        assert!(text.contains("│  ⎿  Reading files"), "canvas=\n{text}");
    }

    #[test]
    fn agent_progress_line_hides_type_and_background_status_like_official() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                AgentProgressLine(
                    agent_type: "worker".to_string(),
                    name: Some("docs".to_string()),
                    description: Some("summarize docs".to_string()),
                    task_description: Some("Running docs summary".to_string()),
                    tool_use_count: 0usize,
                    is_last: true,
                    is_resolved: true,
                    is_async: true,
                    hide_type: true,
                )
            }
        }
        .render(Some(120))
        .to_string();

        assert!(text.contains("└─ docs: summarize docs"), "canvas=\n{text}");
        assert!(!text.contains("tool use"), "canvas=\n{text}");
        assert!(!text.contains("Running docs summary"), "canvas=\n{text}");
    }
}
