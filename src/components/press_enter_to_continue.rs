//! Maps to: CC `components/PressEnterToContinue.tsx`:1-10.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[component]
pub fn PressEnterToContinue(hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = *hooks.use_context::<Theme>();

    element! {
        View(flex_direction: FlexDirection::Row) {
            Text(content: "Press ".to_string(), color: theme.permission, wrap: TextWrap::NoWrap)
            Text(content: "Enter".to_string(), color: theme.permission, weight: Weight::Bold, wrap: TextWrap::NoWrap)
            Text(content: " to continue…".to_string(), color: theme.permission, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn press_enter_to_continue_matches_official_copy() {
        let text = element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                PressEnterToContinue()
            }
        }
        .render(Some(80))
        .to_string();

        assert!(text.contains("Press Enter to continue…"), "canvas=\n{text}");
    }
}
