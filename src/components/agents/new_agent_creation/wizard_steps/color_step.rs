//! Maps to: CC `components/agents/new-agent-creation/wizard-steps/ColorStep.tsx:1-64`.

use crate::components::agents::color_picker::ColorPicker;
use crate::components::agents::new_agent_creation::types::{string, strings};
use crate::components::wizard::{WizardDialogLayout, use_wizard};
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use iocraft::prelude::*;
use serde_json::{Map, Value, json};

#[component]
pub fn ColorStep(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let wizard = use_wizard(&mut hooks);
    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|value| value.clone());
    let back = wizard.clone();
    use_keybinding(
        &mut hooks,
        runtime,
        "confirm:no",
        ContextName::Confirmation,
        || true,
        move || {
            back.go_back();
            true
        },
    );
    let agent_type =
        string(&wizard.wizard_data, "agentType").unwrap_or_else(|| "agent".to_string());
    let when_to_use = string(&wizard.wizard_data, "whenToUse").unwrap_or_default();
    let system_prompt = string(&wizard.wizard_data, "systemPrompt").unwrap_or_default();
    let model = string(&wizard.wizard_data, "selectedModel");
    let location = string(&wizard.wizard_data, "location").unwrap_or_default();
    let tools = strings(&wizard.wizard_data, "selectedTools");
    let complete = wizard.clone();
    let display_name = agent_type.clone();
    element! {
        WizardDialogLayout(
            subtitle: Some("Choose background color".to_string()),
            footer_text: Some("↑↓ to navigate · Enter to select · Esc to go back".to_string()),
        ) {
            ColorPicker(
                agent_name: display_name,
                on_confirm: move |color: Option<crate::tools::agent_tool::agent_color_manager::AgentColorName>| {
                    let color = color.map(|value| value.official_name().to_string());
                    let mut final_agent = Map::new();
                    final_agent.insert("agentType".to_string(), json!(agent_type));
                    final_agent.insert("whenToUse".to_string(), json!(when_to_use));
                    final_agent.insert("systemPrompt".to_string(), json!(system_prompt));
                    final_agent.insert("source".to_string(), json!(location));
                    if let Some(tools) = &tools { final_agent.insert("tools".to_string(), json!(tools)); }
                    if let Some(model) = &model { final_agent.insert("model".to_string(), json!(model)); }
                    if let Some(color) = &color { final_agent.insert("color".to_string(), json!(color)); }
                    let mut data = Map::new();
                    data.insert("selectedColor".to_string(), json!(color));
                    data.insert("finalAgent".to_string(), Value::Object(final_agent));
                    complete.update_wizard_data(data);
                    complete.go_next();
                },
            )
        }
    }
}
