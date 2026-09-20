//! Maps to: CC `components/agents/new-agent-creation/wizard-steps/TypeStep.tsx:1-83`.

use crate::components::agents::new_agent_creation::types::{string, update};
use crate::components::agents::validate_agent::validate_agent_type;
use crate::components::text_input::TextInput;
use crate::components::wizard::{WizardDialogLayout, use_wizard};
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use crate::tools::agent_tool::load_agents_dir::AgentDefinition;
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use serde_json::json;

#[derive(Default, Props)]
pub struct TypeStepProps {
    pub existing_agents: Vec<AgentDefinition>,
}

#[component]
pub fn TypeStep(props: &TypeStepProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let wizard = use_wizard(&mut hooks);
    let initial = string(&wizard.wizard_data, "agentType").unwrap_or_default();
    let initial_len = initial.len();
    let value = hooks.use_state(move || initial);
    let cursor = hooks.use_state(move || initial_len);
    let mut error = hooks.use_state(|| None::<String>);
    let mut submit = hooks.use_state(|| None::<String>);
    let submitted = { submit.read().clone() };
    if let Some(submitted) = submitted {
        submit.set(None);
        let trimmed = submitted.trim().to_string();
        if let Some(message) = validate_agent_type(&trimmed) {
            error.set(Some(message));
        } else {
            error.set(None);
            wizard.update_wizard_data(update("agentType", json!(trimmed)));
            wizard.go_next();
        }
    }
    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|value| value.clone());
    let back = wizard.clone();
    use_keybinding(
        &mut hooks,
        runtime,
        "confirm:no",
        ContextName::Settings,
        || true,
        move || {
            back.go_back();
            true
        },
    );
    let theme = hooks.use_context::<Theme>();
    let mut submit_handler = submit;
    let _ = props.existing_agents.len();
    element! {
        WizardDialogLayout(
            subtitle: Some("Agent type (identifier)".to_string()),
            footer_text: Some("Type to enter text · Enter to continue · Esc to go back".to_string()),
        ) {
            View(flex_direction: FlexDirection::Column) {
                Text(content: "Enter a unique identifier for your agent:".to_string())
                View(margin_top: 1u32) {
                    TextInput(
                        value: value, cursor_offset: cursor, focus: true, show_cursor: true, columns: 60usize,
                        placeholder: Some("e.g., test-runner, tech-lead, etc".to_string()),
                        on_submit: move |text| submit_handler.set(Some(text)),
                    )
                }
                #(error.read().clone().map(|message| element! { View(margin_top: 1u32) { Text(content: message, color: theme.error) } }))
            }
        }
    }
}
