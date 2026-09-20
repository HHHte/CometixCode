//! Maps to: CC `components/agents/new-agent-creation/wizard-steps/GenerateStep.tsx:1-201`.
//!
//! Model work is supplied through `AgentGeneratorRuntime`; the external build
//! does not silently initiate an API request. Cancellation drops the in-flight
//! future and also notifies cancellation-aware injected backends.

use crate::components::agents::generate_agent::{
    AgentGenerationCancellation, AgentGeneratorRuntime, GeneratedAgent,
};
use crate::components::agents::new_agent_creation::types::string;
use crate::components::spinner::Spinner;
use crate::components::text_input::TextInput;
use crate::components::wizard::{WizardDialogLayout, use_wizard};
use crate::keybindings::keybinding_context::KeybindingRuntime;
use crate::keybindings::types::ContextName;
use crate::keybindings::use_keybinding::use_keybinding;
use crate::utils::prompt_editor::{EditorResult, ExternalEditorRuntime};
use crate::utils::theme::Theme;
use iocraft::prelude::*;
use serde_json::{Map, json};
use std::sync::Arc;

struct GenerationRequest {
    id: u64,
    prompt: String,
    cancellation: AgentGenerationCancellation,
}

type GenerationResult = (u64, Result<GeneratedAgent, String>);

#[component]
pub fn GenerateStep(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let wizard = use_wizard(&mut hooks);
    let initial = string(&wizard.wizard_data, "generationPrompt").unwrap_or_default();
    let initial_len = initial.len();
    let mut prompt = hooks.use_state(move || initial);
    let mut cursor = hooks.use_state(move || initial_len);
    let mut is_generating = hooks.use_state(|| false);
    let mut error = hooks.use_state(|| None::<String>);
    let mut pending_submit = hooks.use_state(|| None::<String>);
    let mut completed = hooks.use_state(|| None::<GenerationResult>);
    let mut editor_result = hooks.use_state(|| None::<EditorResult>);
    let mut generation_counter = hooks.use_state(|| 0u64);
    let mut active_generation = hooks.use_state(|| None::<(u64, AgentGenerationCancellation)>);
    let generator = hooks
        .try_use_context::<AgentGeneratorRuntime>()
        .map(|runtime| runtime.clone());
    let editor_runtime = hooks
        .try_use_context::<ExternalEditorRuntime>()
        .map(|runtime| *runtime);
    let channel = hooks.use_const(|| Arc::new(async_channel::unbounded::<GenerationRequest>()));
    let receiver = channel.1.clone();
    hooks.use_future(async move {
        while let Ok(request) = receiver.recv().await {
            let result = match &generator {
                Some(generator) => {
                    generator
                        .generate_with_cancellation(request.prompt, request.cancellation)
                        .await
                }
                None => Err("Agent generation is unavailable in this build".to_string()),
            };
            completed.set(Some((request.id, result)));
        }
    });
    let editor_channel = hooks.use_const(|| Arc::new(async_channel::unbounded::<String>()));
    let editor_receiver = editor_channel.1.clone();
    hooks.use_future(async move {
        while let Ok(content) = editor_receiver.recv().await {
            let result = match editor_runtime {
                Some(runtime) => runtime.edit_prompt(&content).await,
                None => EditorResult {
                    content: None,
                    error: Some("External editor is unavailable".to_string()),
                },
            };
            editor_result.set(Some(result));
        }
    });
    let completed_editor = { editor_result.read().clone() };
    if let Some(result) = completed_editor {
        editor_result.set(None);
        if let Some(content) = result.content {
            cursor.set(content.len());
            prompt.set(content);
        }
        if let Some(message) = result.error {
            error.set(Some(message));
        }
    }

    let submission = { pending_submit.read().clone() };
    if let Some(submission) = submission {
        pending_submit.set(None);
        let trimmed = submission.trim().to_string();
        if trimmed.is_empty() {
            error.set(Some("Please describe what the agent should do".to_string()));
        } else {
            error.set(None);
            is_generating.set(true);
            let mut data = Map::new();
            data.insert("generationPrompt".to_string(), json!(trimmed.clone()));
            data.insert("isGenerating".to_string(), json!(true));
            wizard.update_wizard_data(data);
            let id = generation_counter.get().wrapping_add(1);
            generation_counter.set(id);
            let cancellation = AgentGenerationCancellation::new();
            active_generation.set(Some((id, cancellation.clone())));
            let _ = channel.0.try_send(GenerationRequest {
                id,
                prompt: trimmed,
                cancellation,
            });
        }
    }
    let result = { completed.read().clone() };
    if let Some((id, result)) = result {
        completed.set(None);
        let is_current = active_generation
            .read()
            .as_ref()
            .is_some_and(|(active_id, _)| *active_id == id);
        if is_generating.get() && is_current {
            is_generating.set(false);
            active_generation.set(None);
            let mut data = Map::new();
            data.insert("isGenerating".to_string(), json!(false));
            match result {
                Ok(generated) => {
                    data.insert("agentType".to_string(), json!(generated.identifier));
                    data.insert("whenToUse".to_string(), json!(generated.when_to_use));
                    data.insert("systemPrompt".to_string(), json!(generated.system_prompt));
                    data.insert("generatedAgent".to_string(), json!(generated));
                    data.insert("wasGenerated".to_string(), json!(true));
                    wizard.update_wizard_data(data);
                    wizard.go_to_step(6);
                }
                Err(message) => {
                    error.set(Some(message));
                    wizard.update_wizard_data(data);
                }
            }
        }
    }

    let runtime = hooks
        .try_use_context::<KeybindingRuntime>()
        .map(|value| value.clone());
    let mut generating_for_cancel = is_generating;
    let mut error_for_cancel = error;
    let mut active_for_cancel = active_generation;
    use_keybinding(
        &mut hooks,
        runtime.clone(),
        "confirm:no",
        ContextName::Settings,
        move || is_generating.get(),
        move || {
            if let Some((_id, cancellation)) = active_for_cancel.read().as_ref() {
                cancellation.cancel();
            }
            active_for_cancel.set(None);
            generating_for_cancel.set(false);
            error_for_cancel.set(Some("Generation cancelled".to_string()));
            true
        },
    );
    let back = wizard.clone();
    let mut prompt_for_back = prompt;
    let mut error_for_back = error;
    use_keybinding(
        &mut hooks,
        runtime.clone(),
        "confirm:no",
        ContextName::Settings,
        move || !is_generating.get(),
        move || {
            let mut data = Map::new();
            for (key, value) in [
                ("generationPrompt", json!("")),
                ("agentType", json!("")),
                ("systemPrompt", json!("")),
                ("whenToUse", json!("")),
                ("generatedAgent", serde_json::Value::Null),
                ("wasGenerated", json!(false)),
            ] {
                data.insert(key.to_string(), value);
            }
            back.update_wizard_data(data);
            prompt_for_back.set(String::new());
            error_for_back.set(None);
            back.go_back();
            true
        },
    );

    let prompt_for_editor = prompt;
    let editor_sender = editor_channel.0.clone();
    use_keybinding(
        &mut hooks,
        runtime,
        "chat:externalEditor",
        ContextName::Chat,
        move || !is_generating.get(),
        move || {
            let _ = editor_sender.try_send(prompt_for_editor.read().clone());
            true
        },
    );

    let theme = hooks.use_context::<Theme>();
    let subtitle = "Describe what this agent should do and when it should be used (be comprehensive for best results)".to_string();
    if is_generating.get() {
        return element! {
            WizardDialogLayout(subtitle: Some(subtitle), footer_text: Some("Esc to cancel".to_string())) {
                View(flex_direction: FlexDirection::Row, align_items: AlignItems::CENTER) {
                    Spinner()
                    Text(content: " Generating agent from description...".to_string(), color: theme.suggestion)
                }
            }
        }.into_any();
    }
    let mut submit_handler = pending_submit;
    element! {
        WizardDialogLayout(subtitle: Some(subtitle), footer_text: Some("Enter to submit · Ctrl+G to open in editor · Esc to go back".to_string())) {
            View(flex_direction: FlexDirection::Column) {
                #(error.read().clone().map(|message| element! { View(margin_bottom: 1u32) { Text(content: message, color: theme.error) } }))
                TextInput(
                    value: prompt, cursor_offset: cursor, focus: true, show_cursor: true, columns: 80usize,
                    placeholder: Some("e.g., Help me write unit tests for my code...".to_string()),
                    on_submit: move |text| submit_handler.set(Some(text)),
                )
            }
        }
    }.into_any()
}
