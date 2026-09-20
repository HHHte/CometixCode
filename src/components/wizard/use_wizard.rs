//! Maps to: CC `components/wizard/useWizard.ts:1-13`.

use super::types::WizardContextValue;
use iocraft::prelude::*;

/// Panics outside `WizardProvider`, matching the official hook error contract.
pub fn use_wizard(hooks: &mut Hooks) -> WizardContextValue {
    hooks
        .try_use_context::<WizardContextValue>()
        .map(|context| context.clone())
        .unwrap_or_else(|| panic!("useWizard must be used within a WizardProvider"))
}
