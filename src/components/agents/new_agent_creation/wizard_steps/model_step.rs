//! Maps to: CC `components/agents/new-agent-creation/wizard-steps/ModelStep.tsx:1-42`.

use crate::components::agents::model_selector::ModelSelector;
use crate::components::agents::new_agent_creation::types::{string, update};
use crate::components::wizard::{WizardDialogLayout, use_wizard};
use iocraft::prelude::*;
use serde_json::json;

#[component]
pub fn ModelStep(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let wizard = use_wizard(&mut hooks);
    let initial = string(&wizard.wizard_data, "selectedModel");
    let complete = wizard.clone();
    let back = wizard.clone();
    element! {
        WizardDialogLayout(
            subtitle: Some("Select model".to_string()),
            footer_text: Some("↑↓ to navigate · Enter to select · Esc to go back".to_string()),
        ) {
            ModelSelector(
                initial_model: initial,
                on_complete: move |model| { complete.update_wizard_data(update("selectedModel", json!(model))); complete.go_next(); },
                on_cancel: move |_| back.go_back(),
            )
        }
    }
}
