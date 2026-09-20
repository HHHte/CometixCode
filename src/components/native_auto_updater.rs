//! Maps to: CC `components/NativeAutoUpdater.tsx`.
//!
//! Native-installer auto-updater child. Slice 3b: this component owns its own
//! display state (`versions`, `maxVersionIssue`) and its own check loop
//! (mount check + 30-minute interval), exactly like CC (`:65-71`, `:80-181`).
//! Results and the updating flag travel upward through the CC callback props
//! (`onAutoUpdaterResult` → REPL state, `onChangeIsUpdating` → PromptInput
//! state) — never through AppStore. The native download/install IO is
//! short-circuited in [`crate::utils::auto_updater`]; analytics (`logEvent`)
//! are omitted.

use crate::components::auto_updater::IsUpdatingClearGuard;
use crate::utils::auto_updater::{
    AutoUpdaterResult, CHECK_INTERVAL, InstallStatus, ReleaseChannel, check_for_native_updates,
    is_test_or_dev_env,
};
use crate::utils::config::is_auto_updater_disabled;
use crate::utils::debug::log_for_debugging;
use crate::utils::theme::Theme;
use iocraft::prelude::*;

/// Maps to: CC `NativeAutoUpdater.tsx:223` `"external" === 'ant'` build gate.
/// L1 `Compile-time distribution capability projection` (PORTING.md): audience
/// gates go through `build_profile::has_internal_capability`, never scattered
/// consumer-file flags (precedent: memory_usage_indicator.rs).
fn is_ant_build() -> bool {
    crate::utils::build_profile::has_internal_capability(
        crate::utils::build_profile::InternalCapability::Ui,
    )
}

/// Maps to: CC `NativeAutoUpdater.tsx:48-55` Props — exactly the six CC
/// fields (identical shape for all three updater children and the wrapper).
#[derive(Default, Props)]
pub struct NativeAutoUpdaterProps {
    /// Maps to: CC `isUpdating` (PromptInput-owned `isAutoUpdating`).
    pub is_updating: bool,
    /// Maps to: CC `onChangeIsUpdating` (PromptInput `setIsAutoUpdating`).
    pub on_change_is_updating: Handler<bool>,
    /// Maps to: CC `onAutoUpdaterResult` (REPL `setAutoUpdaterResult`).
    pub on_auto_updater_result: Handler<AutoUpdaterResult>,
    /// Maps to: CC `autoUpdaterResult` (REPL-owned state).
    pub auto_updater_result: Option<AutoUpdaterResult>,
    pub show_success_message: bool,
    pub verbose: bool,
}

/// Maps to: CC `components/NativeAutoUpdater.tsx` `getErrorType(errorMessage)`.
pub fn get_error_type(error_message: &str) -> &'static str {
    if error_message.contains("timeout") {
        "timeout"
    } else if error_message.contains("Checksum mismatch") {
        "checksum_mismatch"
    } else if error_message.contains("ENOENT") || error_message.contains("not found") {
        "not_found"
    } else if error_message.contains("EACCES") || error_message.contains("permission") {
        "permission_denied"
    } else if error_message.contains("ENOSPC") {
        "disk_full"
    } else if error_message.contains("npm") {
        "npm_error"
    } else if error_message.contains("network")
        || error_message.contains("ECONNREFUSED")
        || error_message.contains("ENOTFOUND")
    {
        "network_error"
    } else {
        "unknown"
    }
}

/// Maps to: CC `NativeAutoUpdater.tsx:183-194` `shouldRender`.
pub fn native_auto_updater_should_render(
    max_version_issue_present: bool,
    result_version_present: bool,
    is_updating: bool,
    has_version_info: bool,
) -> bool {
    max_version_issue_present || result_version_present || (is_updating && has_version_info)
}

#[component]
pub fn NativeAutoUpdater(
    props: &NativeAutoUpdaterProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<Theme>();
    // Maps to: CC NativeAutoUpdater.tsx:65-68 `versions` ({current, latest}) —
    // component-local state written only by this child's check loop.
    let versions = hooks.use_state(|| (Option::<String>::None, Option::<String>::None));
    // Maps to: CC NativeAutoUpdater.tsx:69 `maxVersionIssue`.
    let max_version_issue = hooks.use_state(|| Option::<String>::None);
    // Maps to: CC NativeAutoUpdater.tsx:70 `useUpdateNotification(autoUpdaterResult?.version)`
    // — per-instance lastNotifiedSemver state, called before any early return.
    let update_semver = crate::hooks::use_update_notification::use_update_notification(
        &mut hooks,
        props
            .auto_updater_result
            .as_ref()
            .and_then(|result| result.version.as_deref()),
    );
    // Maps to: CC NativeAutoUpdater.tsx:71 `const channel =
    // getInitialSettings()?.autoUpdatesChannel ?? 'latest'` — a local const
    // from initial settings, deliberately not state.
    let channel = hooks.use_const(|| {
        ReleaseChannel::from_settings(
            crate::utils::settings::get_initial_settings()
                .auto_updates_channel
                .as_deref(),
        )
        .as_str()
        .to_string()
    });

    // Maps to: CC NativeAutoUpdater.tsx:80-167 `checkForUpdates` +
    // initial-check effect (:176-178) + `useInterval(30m)` (:180-181) — this
    // child owns its own check loop (slice-3b split). CC's `isUpdatingRef`
    // guard (:73-83) is subsumed by this single sequential loop (see the
    // matching note in components/auto_updater.rs).
    // Skipped in unit tests so canvases stay deterministic.
    hooks.use_future({
        let mut versions = versions;
        let mut max_version_issue = max_version_issue;
        let on_auto_updater_result = props.on_auto_updater_result.clone();
        let on_change_is_updating = props.on_change_is_updating.clone();
        async move {
            if cfg!(test) {
                return;
            }
            loop {
                // Maps to: CC :85-97 guards. A skipped check still sleeps and
                // retries — CC's interval keeps firing through skips too.
                if is_test_or_dev_env() {
                    log_for_debugging(
                        "NativeAutoUpdater: Skipping update check in test/dev environment",
                    );
                } else if !is_auto_updater_disabled() {
                    // Maps to: CC :99 `onChangeIsUpdating(true)` → :165-167
                    // `finally { onChangeIsUpdating(false) }` (clear-on-drop
                    // guard covers the unmount-cancel path — see
                    // IsUpdatingClearGuard).
                    let guard = IsUpdatingClearGuard::arm(&on_change_is_updating);
                    let outcome = check_for_native_updates().await;
                    // Maps to: CC :106-111 `setMaxVersionIssue` — committed
                    // regardless of the lock-contention skip below.
                    if let Some(issue) = outcome.max_version_issue.clone() {
                        max_version_issue.set(Some(issue));
                    }
                    // Maps to: CC :118-122 lock contention — silently skip
                    // commits; the flag still clears via `finally`.
                    if !outcome.install.lock_failed {
                        if let Some(error) = outcome.install.error.clone() {
                            // Maps to: CC :143-164 catch —
                            // `{version: null, status: 'install_failed'}`; the
                            // error path never reaches setVersions or the
                            // success emission (mutually exclusive).
                            log_for_debugging(&format!("NativeAutoUpdater: {error}"));
                            on_auto_updater_result(AutoUpdaterResult {
                                version: None,
                                status: InstallStatus::InstallFailed,
                                notifications: Vec::new(),
                            });
                        } else {
                            // Maps to: CC :126 `setVersions({current, latest})`.
                            versions.set((
                                Some(outcome.current_version.clone()),
                                outcome.install.latest_version.clone(),
                            ));
                            if outcome.install.was_updated {
                                // Maps to: CC :128-136 — emits unconditionally
                                // on the success path (version may be null).
                                on_auto_updater_result(AutoUpdaterResult {
                                    version: outcome.install.latest_version.clone(),
                                    status: InstallStatus::Success,
                                    notifications: Vec::new(),
                                });
                            }
                        }
                    }
                    guard.finish();
                }
                futures_timer::Delay::new(CHECK_INTERVAL).await;
            }
        }
    });

    let (current_version, latest_version) = versions.read().clone();
    let known_issue = max_version_issue.read().clone();
    let result_version_present = props
        .auto_updater_result
        .as_ref()
        .and_then(|result| result.version.as_deref())
        .is_some();
    let has_version_info = current_version.is_some() && latest_version.is_some();
    if !native_auto_updater_should_render(
        known_issue.is_some(),
        result_version_present,
        props.is_updating,
        has_version_info,
    ) {
        return element! { View(width: 0u32, height: 0u32) };
    }

    let result_status = props
        .auto_updater_result
        .as_ref()
        .map(|result| result.status);
    // Maps to: CC :210-212 — `updateSemver` is committed-null as executed
    // (see hooks/use_update_notification.rs), so this row never commits; the
    // expression keeps CC's shape.
    let show_success = result_status == Some(InstallStatus::Success)
        && props.show_success_message
        && update_semver.is_some();
    let show_failure = result_status == Some(InstallStatus::InstallFailed);
    // Maps to: CC :223 — external builds DCE the known-issue row.
    let ant_known_issue = if is_ant_build() { known_issue } else { None };

    element! {
        View(flex_direction: FlexDirection::Row, column_gap: 1u32) {
            #(if props.verbose {
                Some(element! {
                    Text(
                        content: format!(
                            "current: {} · {channel}: {}",
                            current_version.as_deref().unwrap_or(""),
                            latest_version.as_deref().unwrap_or(""),
                        ),
                        color: theme.inactive,
                        dim: true,
                        wrap: TextWrap::Truncate,
                    )
                })
            } else {
                None
            })
            // Maps to: CC NativeAutoUpdater.tsx:203 — plain `isUpdating ?` ternary.
            #(if props.is_updating {
                Some(element! {
                    View {
                        Text(content: "Checking for updates".to_string(), color: theme.inactive, dim: true, wrap: TextWrap::Truncate)
                    }
                }.into_any())
            } else if show_success {
                Some(element! {
                    Text(content: "✓ Update installed · Restart to update".to_string(), color: theme.success, wrap: TextWrap::Truncate)
                }.into_any())
            } else {
                None
            })
            #(if show_failure {
                Some(element! {
                    Text(content: "✗ Auto-update failed · Try /status".to_string(), color: theme.error, wrap: TextWrap::Truncate)
                }.into_any())
            } else {
                None
            })
            #(ant_known_issue.map(|issue| element! {
                View(flex_direction: FlexDirection::Row) {
                    Text(content: format!("⚠ Known issue: {issue} · Run "), color: theme.warning, wrap: TextWrap::Truncate)
                    Text(content: "claude rollback --safe".to_string(), color: theme.warning, weight: Weight::Bold, wrap: TextWrap::Truncate)
                    Text(content: " to downgrade".to_string(), color: theme.warning, wrap: TextWrap::Truncate)
                }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render(props: NativeAutoUpdaterProps) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                NativeAutoUpdater(
                    is_updating: props.is_updating,
                    auto_updater_result: props.auto_updater_result,
                    show_success_message: props.show_success_message,
                    verbose: props.verbose,
                )
            }
        }
        .render(Some(180))
        .to_string()
    }

    #[test]
    fn native_auto_updater_error_type_matches_official_order() {
        assert_eq!(get_error_type("operation timeout"), "timeout");
        assert_eq!(
            get_error_type("Checksum mismatch for archive"),
            "checksum_mismatch"
        );
        assert_eq!(get_error_type("ENOENT missing binary"), "not_found");
        assert_eq!(
            get_error_type("EACCES permission denied"),
            "permission_denied"
        );
        assert_eq!(get_error_type("ENOSPC no space left"), "disk_full");
        assert_eq!(get_error_type("npm failed"), "npm_error");
        assert_eq!(get_error_type("network ECONNREFUSED"), "network_error");
        assert_eq!(get_error_type("something else"), "unknown");
    }

    #[test]
    fn native_auto_updater_render_gate_matches_official_should_render() {
        assert!(!native_auto_updater_should_render(
            false, false, false, false
        ));
        assert!(native_auto_updater_should_render(false, false, true, true));
        assert!(!native_auto_updater_should_render(
            false, false, true, false
        ));
        assert!(native_auto_updater_should_render(true, false, false, false));
        assert!(native_auto_updater_should_render(false, true, false, false));
    }

    #[test]
    fn native_auto_updater_renders_checking_and_failure_copy() {
        // Gate passes via the result version (child-local versions state stays
        // at its CC initial `{}` in tests — the check loop is test-skipped);
        // CC :203 shows the checking row on a plain `isUpdating ?` ternary.
        let checking = render(NativeAutoUpdaterProps {
            is_updating: true,
            verbose: true,
            auto_updater_result: Some(AutoUpdaterResult {
                version: Some("1.1.0".to_string()),
                status: InstallStatus::InProgress,
                notifications: Vec::new(),
            }),
            ..NativeAutoUpdaterProps::default()
        });
        assert!(checking.contains("current:"), "canvas=\n{checking}");
        assert!(
            checking.contains("Checking for updates"),
            "canvas=\n{checking}"
        );

        let failure = render(NativeAutoUpdaterProps {
            auto_updater_result: Some(AutoUpdaterResult {
                version: Some("1.1.0".to_string()),
                status: InstallStatus::InstallFailed,
                notifications: Vec::new(),
            }),
            ..NativeAutoUpdaterProps::default()
        });
        assert!(
            failure.contains("✗ Auto-update failed · Try /status"),
            "canvas=\n{failure}"
        );
    }

    /// Maps to: CC NativeAutoUpdater.tsx:210-216 as executed —
    /// `useUpdateNotification` committed output is always null (render-phase
    /// set + React discard), so the success copy never commits.
    #[test]
    fn native_auto_updater_success_copy_never_commits_matching_official_render_discard() {
        let success = render(NativeAutoUpdaterProps {
            auto_updater_result: Some(AutoUpdaterResult {
                version: Some("1.1.0".to_string()),
                status: InstallStatus::Success,
                notifications: Vec::new(),
            }),
            show_success_message: true,
            ..NativeAutoUpdaterProps::default()
        });
        assert!(
            !success.contains("✓ Update installed"),
            "canvas=\n{success}"
        );
    }
}
