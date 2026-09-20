//! Maps to: CC `components/agents/new-agent-creation/wizard-steps/ToolsStep.tsx:1-52`.

use crate::components::agents::new_agent_creation::types::{strings, update};
use crate::components::agents::tool_selector::{AgentToolOption, ToolSelector};
use crate::components::wizard::{WizardDialogLayout, use_wizard};
use iocraft::prelude::*;
use serde_json::json;

#[derive(Default, Props)]
pub struct ToolsStepProps {
    pub tools: Vec<AgentToolOption>,
}

#[component]
pub fn ToolsStep(props: &ToolsStepProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let wizard = use_wizard(&mut hooks);
    let initial = strings(&wizard.wizard_data, "selectedTools");
    let complete = wizard.clone();
    let back = wizard.clone();
    element! {
        WizardDialogLayout(
            subtitle: Some("Select tools".to_string()),
            footer_text: Some("Enter to toggle selection · ↑↓ to navigate · Esc to go back".to_string()),
        ) {
            ToolSelector(
                tools: props.tools.clone(), initial_tools: initial,
                on_complete: move |tools| { complete.update_wizard_data(update("selectedTools", json!(tools))); complete.go_next(); },
                on_cancel: move |_| back.go_back(),
            )
        }
    }
}
