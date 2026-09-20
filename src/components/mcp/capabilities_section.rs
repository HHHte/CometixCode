//! Maps to: CC `components/mcp/CapabilitiesSection.tsx`.

use crate::components::design_system::byline::Byline;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct CapabilitiesSectionProps {
    pub server_tools_count: usize,
    pub server_prompts_count: usize,
    pub server_resources_count: usize,
}

pub fn capability_labels(tools: usize, prompts: usize, resources: usize) -> Vec<&'static str> {
    let mut capabilities = Vec::new();
    if tools > 0 {
        capabilities.push("tools");
    }
    if resources > 0 {
        capabilities.push("resources");
    }
    if prompts > 0 {
        capabilities.push("prompts");
    }
    capabilities
}

#[component]
pub fn CapabilitiesSection(
    props: &CapabilitiesSectionProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks
        .try_use_context::<crate::utils::theme::Theme>()
        .map(|theme| *theme)
        .unwrap_or_else(|| *crate::utils::theme::current());
    let capabilities = capability_labels(
        props.server_tools_count,
        props.server_prompts_count,
        props.server_resources_count,
    );

    element! {
        View(flex_direction: FlexDirection::Row) {
            Text(content: "Capabilities: ".to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
            #(if capabilities.is_empty() {
                Some(element! { Text(content: "none".to_string(), color: theme.text, wrap: TextWrap::NoWrap) }.into_any())
            } else {
                Some(element! {
                    Byline {
                        #(capabilities.into_iter().map(|capability| element! { Text(content: capability.to_string(), color: theme.text, wrap: TextWrap::NoWrap) }))
                    }
                }.into_any())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_labels_match_official_order() {
        assert_eq!(
            capability_labels(1, 1, 1),
            vec!["tools", "resources", "prompts"]
        );
        assert!(capability_labels(0, 0, 0).is_empty());
    }
}
