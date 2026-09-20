//! Maps to: CC `components/CompactSummary.tsx`.

use crate::components::configurable_shortcut_hint::ConfigurableShortcutHint;
use crate::components::message_response::MessageResponse;
use crate::constants::figures::BLACK_CIRCLE;
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompactSummaryScreen {
    #[default]
    Main,
    Transcript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactSummaryDirection {
    UpTo,
    From,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompactSummaryMetadata {
    pub messages_summarized: usize,
    pub direction: CompactSummaryDirection,
    pub user_context: Option<String>,
}

#[derive(Default, Props)]
pub struct CompactSummaryProps {
    pub text_content: String,
    pub screen: CompactSummaryScreen,
    pub metadata: Option<CompactSummaryMetadata>,
}

/// Maps to: CC `components/CompactSummary.tsx` metadata detail text.
pub fn compact_summary_metadata_text(metadata: &CompactSummaryMetadata) -> String {
    format!(
        "Summarized {} messages {}",
        metadata.messages_summarized,
        match metadata.direction {
            CompactSummaryDirection::UpTo => "up to this point",
            CompactSummaryDirection::From => "from this point",
        }
    )
}

/// Maps to: CC `components/CompactSummary.tsx#CompactSummary`.
#[component]
pub fn CompactSummary(props: &CompactSummaryProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let is_transcript_mode = props.screen == CompactSummaryScreen::Transcript;
    let text_content = props.text_content.clone();
    let metadata = props.metadata.clone();

    if let Some(metadata) = metadata {
        let metadata_text = compact_summary_metadata_text(&metadata);
        let user_context = metadata.user_context.clone();
        return element! {
            View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
                View(flex_direction: FlexDirection::Row) {
                    View(min_width: 2u32) {
                        Text(content: BLACK_CIRCLE.to_string(), color: theme.text, wrap: TextWrap::NoWrap)
                    }
                    View(flex_direction: FlexDirection::Column) {
                        Text(content: "Summarized conversation".to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
                        #(if !is_transcript_mode {
                            element! {
                                MessageResponse {
                                    View(flex_direction: FlexDirection::Column) {
                                        Text(content: metadata_text, dim: true, wrap: TextWrap::Wrap)
                                        #(user_context.map(|context| element! {
                                            Text(content: format!("Context: “{context}”"), dim: true, wrap: TextWrap::Wrap)
                                        }))
                                        View(flex_direction: FlexDirection::Row) {
                                            ConfigurableShortcutHint(
                                                action: "app:toggleTranscript".to_string(),
                                                context: "Global".to_string(),
                                                fallback: "ctrl+o".to_string(),
                                                description: "expand history".to_string(),
                                                parens: true,
                                            )
                                        }
                                    }
                                }
                            }.into_any()
                        } else {
                            element! {
                                MessageResponse {
                                    Text(content: text_content.clone(), wrap: TextWrap::Wrap)
                                }
                            }.into_any()
                        })
                    }
                }
            }
        }
        .into_any();
    }

    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            View(flex_direction: FlexDirection::Row) {
                View(min_width: 2u32) {
                    Text(content: BLACK_CIRCLE.to_string(), color: theme.text, wrap: TextWrap::NoWrap)
                }
                View(flex_direction: FlexDirection::Column) {
                    View(flex_direction: FlexDirection::Row) {
                        Text(content: "Compact summary".to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
                        #(if !is_transcript_mode {
                            Some(element! {
                                View(flex_direction: FlexDirection::Row) {
                                    Text(content: " ".to_string(), dim: true, wrap: TextWrap::NoWrap)
                                    ConfigurableShortcutHint(
                                        action: "app:toggleTranscript".to_string(),
                                        context: "Global".to_string(),
                                        fallback: "ctrl+o".to_string(),
                                        description: "expand".to_string(),
                                        parens: true,
                                    )
                                }
                            })
                        } else {
                            None
                        })
                    }
                }
            }
            #(if is_transcript_mode {
                Some(element! {
                    MessageResponse {
                        Text(content: text_content, wrap: TextWrap::Wrap)
                    }
                })
            } else {
                None
            })
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render(props: CompactSummaryProps) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                CompactSummary(
                    text_content: props.text_content,
                    screen: props.screen,
                    metadata: props.metadata,
                )
            }
        }
        .render(Some(120))
        .to_string()
    }

    #[test]
    fn compact_summary_metadata_text_matches_official_direction_copy() {
        assert_eq!(
            compact_summary_metadata_text(&CompactSummaryMetadata {
                messages_summarized: 4,
                direction: CompactSummaryDirection::UpTo,
                user_context: None,
            }),
            "Summarized 4 messages up to this point"
        );
        assert_eq!(
            compact_summary_metadata_text(&CompactSummaryMetadata {
                messages_summarized: 2,
                direction: CompactSummaryDirection::From,
                user_context: None,
            }),
            "Summarized 2 messages from this point"
        );
    }

    #[test]
    fn compact_summary_metadata_main_screen_renders_official_details() {
        let text = render(CompactSummaryProps {
            text_content: "hidden transcript text".to_string(),
            screen: CompactSummaryScreen::Main,
            metadata: Some(CompactSummaryMetadata {
                messages_summarized: 9,
                direction: CompactSummaryDirection::UpTo,
                user_context: Some("focus area".to_string()),
            }),
        });

        assert!(text.contains("Summarized conversation"), "canvas=\n{text}");
        assert!(
            text.contains("Summarized 9 messages up to this point"),
            "canvas=\n{text}"
        );
        assert!(text.contains("Context: “focus area”"), "canvas=\n{text}");
        assert!(text.contains("ctrl+o to expand history"), "canvas=\n{text}");
        assert!(!text.contains("hidden transcript text"), "canvas=\n{text}");
    }

    #[test]
    fn compact_summary_transcript_mode_shows_message_text() {
        let text = render(CompactSummaryProps {
            text_content: "summary body".to_string(),
            screen: CompactSummaryScreen::Transcript,
            metadata: None,
        });

        assert!(text.contains("Compact summary"), "canvas=\n{text}");
        assert!(text.contains("summary body"), "canvas=\n{text}");
        assert!(!text.contains("ctrl+o"), "canvas=\n{text}");
    }
}
