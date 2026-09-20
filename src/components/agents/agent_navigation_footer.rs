//! Maps to: CC `components/agents/AgentNavigationFooter.tsx:1-23`.

use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct AgentNavigationFooterProps {
    pub instructions: Option<String>,
}

#[component]
pub fn AgentNavigationFooter(
    props: &AgentNavigationFooterProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let exit_state = crate::hooks::use_exit::use_exit_on_ctrl_cd_with_keybindings(&mut hooks, true);
    let text = exit_state.hint().map(str::to_string).unwrap_or_else(|| {
        props.instructions.clone().unwrap_or_else(|| {
            "Press ↑↓ to navigate · Enter to select · Esc to go back".to_string()
        })
    });
    element! {
        View(margin_left: 2u32) {
            Text(content: text, dim: true, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_default_and_custom_copy_are_dim() {
        let canvas = element! { AgentNavigationFooter() }.render(Some(100));
        assert!(
            canvas
                .to_string()
                .contains("  Press ↑↓ to navigate · Enter to select · Esc to go back")
        );
        assert_eq!(
            canvas.resolved_text_style(2, 0).unwrap().weight,
            Weight::Light
        );
        assert!(
            element! { AgentNavigationFooter(instructions: Some("Enter or Esc".to_string())) }
                .render(Some(40))
                .to_string()
                .contains("Enter or Esc")
        );
    }
}
