//! Maps to: CC `components/PromptInput/PromptInputModeIndicator.tsx:1-105`.

use super::input_modes::PromptInputMode;
use crate::tools::agent_tool::agent_color_manager::AgentColorName;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct PromptInputModeIndicatorProps {
    pub mode: PromptInputMode,
    pub is_loading: bool,
    pub viewing_agent_name: Option<String>,
    pub viewing_agent_color: Option<AgentColorName>,
    /// Resolved process teammate color from the swarm context.
    pub teammate_color: Option<AgentColorName>,
}

/// Renders the prompt character only; mode ownership stays in PromptInput.
#[component]
pub fn PromptInputModeIndicator(
    props: &PromptInputModeIndicatorProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    let viewed_color = props.viewing_agent_color.map(|color| {
        crate::utils::iocraft_color::to_iocraft_color(Some(color.official_name()), *theme)
    });
    let teammate_color = props.teammate_color.map(|color| {
        crate::utils::iocraft_color::to_iocraft_color(Some(color.official_name()), *theme)
    });
    let color = if props.viewing_agent_name.is_some() {
        viewed_color
    } else {
        teammate_color
    };

    element! {
        View(
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FLEX_START,
            align_self: AlignSelf::FLEX_START,
            flex_wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FLEX_START,
        ) {
            #(if props.viewing_agent_name.is_none() && props.mode == PromptInputMode::Bash {
                Some(element! {
                    Text(content: "! ".to_string(), color: theme.bash_border, dim: props.is_loading, wrap: TextWrap::NoWrap)
                })
            } else {
                Some(element! {
                    Text(content: "❯ ".to_string(), color: color, dim: props.is_loading, wrap: TextWrap::NoWrap)
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_bash_loading_and_viewed_agent_precedence_match_official() {
        let theme = *crate::utils::theme::current();
        let bash = element! {
            ContextProvider(value: Context::owned(theme)) {
                PromptInputModeIndicator(mode: PromptInputMode::Bash, is_loading: true)
            }
        }
        .render(Some(20));
        assert_eq!(bash.to_string().trim_end(), "!");
        assert_eq!(
            bash.resolved_text_style(0, 0).unwrap().color,
            Some(theme.bash_border)
        );
        assert!(bash.resolved_text_style(0, 0).unwrap().dim);

        let viewed = element! {
            ContextProvider(value: Context::owned(theme)) {
                PromptInputModeIndicator(
                    mode: PromptInputMode::Bash,
                    viewing_agent_name: Some("agent".to_string()),
                    viewing_agent_color: Some(AgentColorName::Purple),
                    teammate_color: Some(AgentColorName::Red),
                )
            }
        }
        .render(Some(20));
        assert_eq!(viewed.to_string().trim_end(), "❯");
        assert_eq!(
            viewed.resolved_text_style(0, 0).unwrap().color,
            Some(theme.agent_purple)
        );
    }
}
