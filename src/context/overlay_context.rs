//! Maps to: CC `context/overlayContext.tsx` — overlay tracking for Escape
//! key coordination, backed by `AppState.activeOverlays`.
//!
//! CancelRequestHandler (keybindings `chat:cancel`) must not abort a running
//! query when the user is merely dismissing an open overlay (Select with
//! onCancel, FuzzyPicker, ...). Overlay components register themselves while
//! mounted; the cancel handler checks `is_overlay_active` in its isActive
//! gate.
//!
//! CC registers/unregisters via useEffect mount/cleanup. iocraft components
//! have no unmount cleanup hook exposed to user code, so registration here
//! is guard-based: `OverlayRegistration` unregisters on Drop, and callers
//! hold it in `use_state` so its lifetime matches the component's.

use std::sync::Arc;

use crate::state::store::{AppStore, UpdateDecision};

/// Maps to: CC `NON_MODAL_OVERLAYS` — overlays that shouldn't disable
/// TextInput focus.
const NON_MODAL_OVERLAYS: &[&str] = &["autocomplete"];

/// RAII registration: the overlay id stays in `AppState.active_overlays`
/// while this guard lives. Maps to: CC `useRegisterOverlay(id, enabled)`
/// effect registration + cleanup.
pub struct OverlayRegistration {
    store: AppStore,
    id: Arc<str>,
}

impl OverlayRegistration {
    pub fn register(store: AppStore, id: &str) -> Self {
        let id: Arc<str> = id.into();
        // P3 §3a: an already-registered id returns `prev` untouched (CC
        // effect no-ops when the Set already holds the id).
        store.set_state(|prev| {
            if prev.active_overlays.contains(id.as_ref()) {
                return UpdateDecision::Same(());
            }
            let mut overlays = (*prev.active_overlays).clone();
            overlays.insert(id.to_string());
            let mut next = (**prev).clone();
            next.active_overlays = Arc::new(overlays);
            UpdateDecision::Replace {
                next: Arc::new(next),
                result: (),
            }
        });
        Self { store, id }
    }
}

impl Drop for OverlayRegistration {
    fn drop(&mut self) {
        // P3 §3a: dropping a guard whose id is already gone (duplicate
        // registration) returns `prev` untouched — Same also keeps this
        // Drop from re-entering the effect pipeline needlessly.
        self.store.set_state(|prev| {
            if !prev.active_overlays.contains(self.id.as_ref()) {
                return UpdateDecision::Same(());
            }
            let mut overlays = (*prev.active_overlays).clone();
            overlays.remove(self.id.as_ref());
            let mut next = (**prev).clone();
            next.active_overlays = Arc::new(overlays);
            UpdateDecision::Replace {
                next: Arc::new(next),
                result: (),
            }
        });
    }
}

/// Maps to: CC `overlayContext.tsx:87-89 useIsOverlayActive()` —
/// `useAppState(s => s.activeOverlays.size > 0)`, i.e. a SUBSCRIBED render-time
/// read. Use this from a component body; [`is_overlay_active`] is the live
/// store read for imperative call sites (keybinding `is_active` closures,
/// evaluated per keystroke rather than per render).
pub fn use_is_overlay_active(hooks: &mut iocraft::prelude::Hooks) -> bool {
    crate::state::app_state::use_app_state(hooks, |state| !state.active_overlays.is_empty())
}

/// Maps to: CC `overlayContext.tsx:102-109 useIsModalOverlayActive()` — the
/// subscribed read of "any overlay that should capture all input" (non-modal
/// overlays like autocomplete don't disable TextInput).
pub fn use_is_modal_overlay_active(hooks: &mut iocraft::prelude::Hooks) -> bool {
    crate::state::app_state::use_app_state(hooks, modal_overlay_active)
}

/// Live store read behind [`use_is_overlay_active`], for imperative callers.
pub fn is_overlay_active(store: &AppStore) -> bool {
    !store.get().active_overlays.is_empty()
}

/// Live store read behind [`use_is_modal_overlay_active`], for imperative
/// callers.
pub fn is_modal_overlay_active(store: &AppStore) -> bool {
    modal_overlay_active(&store.get())
}

fn modal_overlay_active(state: &crate::state::app_state_store::AppState) -> bool {
    state
        .active_overlays
        .iter()
        .any(|id| !NON_MODAL_OVERLAYS.contains(&id.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::app_state_store::AppState;

    #[test]
    fn registration_guard_adds_and_removes_overlay_like_cc_effect_cleanup() {
        let store = AppStore::new(AppState::default(), None);
        assert!(!is_overlay_active(&store));

        {
            let _guard = OverlayRegistration::register(store.clone(), "select");
            assert!(is_overlay_active(&store));
            assert!(is_modal_overlay_active(&store));
        }
        assert!(!is_overlay_active(&store));
    }

    #[test]
    fn autocomplete_is_non_modal_like_cc_allowlist() {
        let store = AppStore::new(AppState::default(), None);
        let _guard = OverlayRegistration::register(store.clone(), "autocomplete");
        assert!(is_overlay_active(&store));
        assert!(!is_modal_overlay_active(&store));
    }

    #[test]
    fn duplicate_registration_is_idempotent() {
        let store = AppStore::new(AppState::default(), None);
        let guard_a = OverlayRegistration::register(store.clone(), "select");
        let guard_b = OverlayRegistration::register(store.clone(), "select");
        drop(guard_a);
        // CC keys by id in a Set: second unregister of the same id is the
        // documented divergence risk; the guard model drops the id as soon
        // as the first guard goes, matching Set semantics.
        assert!(!is_overlay_active(&store));
        drop(guard_b);
        assert!(!is_overlay_active(&store));
    }
}
