//! Maps to: CC `components/HelpV2/Commands.tsx`.

use std::collections::HashSet;
use std::sync::Arc;

use crate::commands::Command;
use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct CommandsProps {
    pub commands: Arc<Vec<Command>>,
    pub max_description_chars: usize,
    pub visible_count: usize,
    pub focused_index: usize,
    pub visible_from_index: usize,
    pub header_focused: bool,
    pub title: String,
    pub empty_message: Option<String>,
}

/// Maps to CC `components/HelpV2/Commands.tsx#Commands`.
#[component]
pub fn Commands(props: &CommandsProps, _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    // Maps to the `useMemo` projection inside canonical `Commands`.
    let mut seen = HashSet::new();
    let mut options = props
        .commands
        .iter()
        .filter(|command| seen.insert(command.name.to_string()))
        .map(|command| SelectOptionData {
            label: format!("/{}", crate::commands::get_command_name(command)),
            value: command.name.to_string(),
            description: Some(crate::utils::truncate::truncate(
                &crate::commands::format_description_with_source(command),
                props.max_description_chars,
                false,
            )),
            dim_description: true,
            disabled: false,
            input: None,
        })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| left.label.cmp(&right.label));

    element! {
        View(flex_direction: FlexDirection::Column, padding_top: 1u32, padding_bottom: 1u32) {
            #(if options.is_empty() && props.empty_message.is_some() {
                element! {
                    Text(content: props.empty_message.clone().unwrap_or_default(), dim: true)
                }.into_any()
            } else {
                element! {
                    Fragment {
                        Text(content: props.title.clone())
                        View(margin_top: 1u32) {
                            Select(
                                options: options,
                                focused_index: props.focused_index,
                                visible_from_index: props.visible_from_index,
                                visible_option_count: props.visible_count,
                                layout: SelectLayout::CompactVertical,
                                hide_indexes: true,
                                is_disabled: props.header_focused,
                            )
                        }
                    }
                }.into_any()
            })
        }
    }
}
