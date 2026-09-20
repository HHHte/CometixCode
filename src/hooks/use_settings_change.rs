//! Maps to: CC `hooks/useSettingsChange.ts`.
//!
//! ```ts
//! export function useSettingsChange(
//!   onChange: (source: SettingSource, settings: SettingsJson) => void,
//! ): void {
//!   const handleChange = useCallback((source: SettingSource) => {
//!     // Cache is already reset by the notifier (changeDetector.fanOut) --
//!     // resetting here caused N-way thrashing with N subscribers.
//!     const newSettings = getSettings_DEPRECATED()
//!     onChange(source, newSettings)
//!   }, [onChange])
//!   useEffect(() => settingsChangeDetector.subscribe(handleChange), [handleChange])
//! }
//! ```
//!
//! The hook owns three things the call site must not re-implement: reading the
//! fresh settings for the subscriber (through the cache the notifier just
//! repopulated — never a second reset here, `:10-13`), the subscription
//! itself, and its cleanup. `AppStateProvider` (`AppState.tsx:104-110`) is
//! CC's only consumer and ignores the settings argument, but the argument is
//! part of the source contract and is passed through.
//!
//! Deviation from CC, deliberate: React's `useEffect` cleanup runs at unmount;
//! iocraft effects have no cleanup, so the subscription is owned by a
//! `use_state`-held RAII guard whose `Drop` unsubscribes when the component's
//! hook storage is reclaimed. Same lifetime, different mechanism.

use iocraft::prelude::*;
use std::sync::Arc;

use crate::utils::settings::change_detector::{self, SettingsChangeUnsubscribe};
use crate::utils::settings::constants::SettingSource;
use crate::utils::settings::types::SettingsJson;

/// RAII subscription, mirroring the cleanup returned by CC's
/// `useEffect(() => settingsChangeDetector.subscribe(handleChange))`.
struct SettingsSubscription(SettingsChangeUnsubscribe);

impl Drop for SettingsSubscription {
    fn drop(&mut self) {
        self.0.unsubscribe();
    }
}

/// Maps to: CC `useSettingsChange(onChange)` (`useSettingsChange.ts:7-25`).
///
/// Subscribes once per mount; the handler receives `(source, settings)` with
/// the settings read through the already-reset cache, exactly like the source.
pub fn use_settings_change(
    hooks: &mut Hooks,
    on_change: impl Fn(SettingSource, SettingsJson) + Send + Sync + 'static,
) {
    hooks.use_state(move || {
        SettingsSubscription(change_detector::subscribe(Arc::new(move |source| {
            // CC :12-15 — the notifier already reset the cache; reading here
            // repopulates it once for every later consumer. Resetting again
            // would restore the N-way thrashing CC removed.
            let settings = crate::utils::settings::get_initial_settings();
            on_change(source, settings);
        })))
    });
}
