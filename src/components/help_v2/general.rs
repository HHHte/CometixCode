//! Maps to: CC `components/HelpV2/General.tsx`.

use crate::components::prompt_input::prompt_input_help_menu::PromptInputHelpMenu;
use iocraft::prelude::*;

/// Maps to CC `components/HelpV2/General.tsx#General`.
#[component]
pub fn General(_hooks: Hooks) -> impl Into<AnyElement<'static>> {
    element! {
        View(flex_direction: FlexDirection::Column, margin_top: 1u32) {
            Text(content: "Cometix Code understands your codebase, makes edits with your permission, and executes commands — right from your terminal.".to_string())
            View(margin_top: 1u32, flex_direction: FlexDirection::Column) {
                Text(content: "Shortcuts".to_string(), weight: Weight::Bold)
                PromptInputHelpMenu(gap: 2u32, fixed_width: true, dim_color: false)
            }
        }
    }
}
