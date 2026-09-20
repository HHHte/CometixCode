//! Runtime types inferred from the non-stub wizard implementation.
//!
//! CC `components/wizard/types.ts` is `@generated-stub`; these shapes map the
//! concrete fields consumed by `WizardProvider.tsx` and `useWizard.ts`.

use iocraft::prelude::{AnyElement, HandlerMut};
use serde_json::{Map, Value};
use std::fmt;
use std::sync::Arc;

pub type WizardData = Map<String, Value>;

#[derive(Clone)]
pub struct WizardStep {
    renderer: Arc<dyn Fn() -> AnyElement<'static> + Send + Sync>,
}

impl WizardStep {
    pub fn new(renderer: impl Fn() -> AnyElement<'static> + Send + Sync + 'static) -> Self {
        Self {
            renderer: Arc::new(renderer),
        }
    }

    pub fn render(&self) -> AnyElement<'static> {
        (self.renderer)()
    }
}

impl fmt::Debug for WizardStep {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("WizardStep").finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WizardMachine {
    pub current_step_index: usize,
    pub total_steps: usize,
    pub wizard_data: WizardData,
    pub navigation_history: Vec<usize>,
    pub is_completed: bool,
}

impl WizardMachine {
    pub fn new(total_steps: usize, initial_data: WizardData) -> Self {
        Self {
            current_step_index: 0,
            total_steps,
            wizard_data: initial_data,
            navigation_history: Vec::new(),
            is_completed: false,
        }
    }

    /// Maps to: CC `WizardProvider.goNext`.
    pub fn go_next(&mut self) -> WizardTransition {
        if self.current_step_index < self.total_steps.saturating_sub(1) {
            if !self.navigation_history.is_empty() {
                self.navigation_history.push(self.current_step_index);
            }
            self.current_step_index += 1;
            WizardTransition::Changed
        } else {
            self.is_completed = true;
            self.navigation_history.clear();
            WizardTransition::Complete(self.wizard_data.clone())
        }
    }

    /// Maps to: CC `WizardProvider.goBack`.
    pub fn go_back(&mut self) -> WizardTransition {
        if let Some(previous) = self.navigation_history.pop() {
            self.current_step_index = previous;
            WizardTransition::Changed
        } else if self.current_step_index > 0 {
            self.current_step_index -= 1;
            WizardTransition::Changed
        } else {
            WizardTransition::Cancel
        }
    }

    /// Maps to: CC `WizardProvider.goToStep`.
    pub fn go_to_step(&mut self, index: usize) -> WizardTransition {
        if index < self.total_steps {
            self.navigation_history.push(self.current_step_index);
            self.current_step_index = index;
            WizardTransition::Changed
        } else {
            WizardTransition::None
        }
    }

    pub fn cancel(&mut self) -> WizardTransition {
        self.navigation_history.clear();
        WizardTransition::Cancel
    }

    /// Maps to the shallow object merge in `updateWizardData`.
    pub fn update_wizard_data(&mut self, updates: WizardData) {
        self.wizard_data.extend(updates);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum WizardTransition {
    None,
    Changed,
    Complete(WizardData),
    Cancel,
}

#[derive(Clone, Debug)]
pub enum WizardAction {
    GoNext,
    GoBack,
    GoToStep(usize),
    Cancel,
    SetData(WizardData),
    UpdateData(WizardData),
}

#[derive(Clone, Debug)]
pub struct WizardContextValue {
    pub current_step_index: usize,
    pub total_steps: usize,
    pub wizard_data: WizardData,
    pub title: Option<String>,
    pub show_step_counter: bool,
    pub(crate) action_sender: async_channel::Sender<WizardAction>,
}

impl WizardContextValue {
    fn send(&self, action: WizardAction) {
        let _ = self.action_sender.try_send(action);
    }

    pub fn go_next(&self) {
        self.send(WizardAction::GoNext);
    }

    pub fn go_back(&self) {
        self.send(WizardAction::GoBack);
    }

    pub fn go_to_step(&self, index: usize) {
        self.send(WizardAction::GoToStep(index));
    }

    pub fn cancel(&self) {
        self.send(WizardAction::Cancel);
    }

    pub fn set_wizard_data(&self, data: WizardData) {
        self.send(WizardAction::SetData(data));
    }

    pub fn update_wizard_data(&self, updates: WizardData) {
        self.send(WizardAction::UpdateData(updates));
    }
}

#[derive(Default, iocraft::Props)]
pub struct WizardProviderProps<'a> {
    pub steps: Vec<WizardStep>,
    pub initial_data: WizardData,
    pub on_complete: HandlerMut<'a, WizardData>,
    pub on_cancel: HandlerMut<'a, ()>,
    pub title: Option<String>,
    pub show_step_counter: Option<bool>,
    /// Rust renderer equivalent of optional official `children`.
    pub child_renderer: Option<WizardStep>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn linear_navigation_and_completion_match_provider() {
        let mut machine = WizardMachine::new(2, WizardData::new());
        assert_eq!(machine.go_next(), WizardTransition::Changed);
        assert_eq!(machine.current_step_index, 1);
        assert!(matches!(machine.go_next(), WizardTransition::Complete(_)));
        assert!(machine.is_completed);
    }

    #[test]
    fn non_linear_history_returns_through_jump_path() {
        let mut machine = WizardMachine::new(6, WizardData::new());
        assert_eq!(machine.go_to_step(3), WizardTransition::Changed);
        assert_eq!(machine.go_next(), WizardTransition::Changed);
        assert_eq!(machine.current_step_index, 4);
        assert_eq!(machine.go_back(), WizardTransition::Changed);
        assert_eq!(machine.current_step_index, 3);
        assert_eq!(machine.go_back(), WizardTransition::Changed);
        assert_eq!(machine.current_step_index, 0);
        assert_eq!(machine.go_back(), WizardTransition::Cancel);
    }

    #[test]
    fn invalid_jump_and_shallow_updates_match_official() {
        let mut data = WizardData::new();
        data.insert("name".to_string(), json!("before"));
        let mut machine = WizardMachine::new(2, data);
        assert_eq!(machine.go_to_step(9), WizardTransition::None);
        let mut updates = WizardData::new();
        updates.insert("name".to_string(), json!("after"));
        updates.insert("color".to_string(), json!("blue"));
        machine.update_wizard_data(updates);
        assert_eq!(machine.wizard_data["name"], json!("after"));
        assert_eq!(machine.wizard_data["color"], json!("blue"));
    }
}
