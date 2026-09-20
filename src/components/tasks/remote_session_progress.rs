//! Maps to: CC `components/tasks/RemoteSessionProgress.tsx:1-201`.

use super::task_status_utils::TaskStatus;
use crate::constants::figures::{DIAMOND_FILLED, DIAMOND_OPEN};
use crate::utils::theme::Theme;
use crate::utils::thinking::get_rainbow_color;
use iocraft::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewStage {
    Finding,
    Verifying,
    Synthesizing,
}

pub fn format_review_stage_counts(
    stage: Option<ReviewStage>,
    found: usize,
    verified: usize,
    refuted: usize,
) -> String {
    match stage {
        None => format!("{found} found · {verified} verified"),
        Some(ReviewStage::Synthesizing) => {
            let mut parts = vec![format!("{verified} verified")];
            if refuted > 0 {
                parts.push(format!("{refuted} refuted"));
            }
            parts.push("deduping".to_string());
            parts.join(" · ")
        }
        Some(ReviewStage::Verifying) => {
            let mut parts = vec![format!("{found} found"), format!("{verified} verified")];
            if refuted > 0 {
                parts.push(format!("{refuted} refuted"));
            }
            parts.join(" · ")
        }
        Some(ReviewStage::Finding) => {
            if found > 0 {
                format!("{found} found")
            } else {
                "finding".to_string()
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemoteSessionProgressData {
    pub status: TaskStatus,
    pub is_remote_review: bool,
    pub review_stage: Option<ReviewStage>,
    pub bugs_found: usize,
    pub bugs_verified: usize,
    pub bugs_refuted: usize,
    pub has_review_progress: bool,
    pub todo_completed: usize,
    pub todo_total: usize,
    pub status_label: Option<String>,
}

#[derive(Default, Props)]
pub struct RemoteSessionProgressProps {
    pub session: Option<RemoteSessionProgressData>,
}

#[component]
pub fn RemoteSessionProgress(
    props: &RemoteSessionProgressProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    // Hooks run before the early return and before the branch: iocraft resolves
    // them by call index, so skipping them when `session` is absent or the
    // session is not a remote review shifts every later hook and panics with
    // "Unexpected hook type!" on the render after the condition flips. The
    // animation stays gated through the period argument — `None` idles the
    // clock — so hoisting costs nothing at runtime.
    let theme = hooks.use_context::<Theme>();
    let reduced = crate::state::app_state::use_app_state(&mut hooks, |state| {
        state.settings.prefers_reduced_motion.unwrap_or(false)
    });
    let running = props.session.as_ref().is_some_and(|session| {
        session.is_remote_review
            && matches!(session.status, TaskStatus::Running | TaskStatus::Pending)
    });
    let frame =
        hooks.use_animation_frame((running && !reduced).then_some(Duration::from_millis(80)));

    let Some(session) = props.session.as_ref() else {
        return element! { Fragment }.into_any();
    };
    if session.is_remote_review {
        let phase = if running && !reduced {
            ((frame.time_ms / 240) as usize) % 7
        } else {
            0
        };
        let rainbow = "ultrareview".chars().enumerate().map(|(index, ch)| element! { Text(content: ch.to_string(), color: get_rainbow_color(&theme, index + phase, false)) }).collect::<Vec<_>>();
        let (diamond, tail, tail_color) = match session.status {
            TaskStatus::Completed => (DIAMOND_FILLED, " ready · shift+↓ to view".to_string(), None),
            TaskStatus::Failed | TaskStatus::Killed => {
                (DIAMOND_FILLED, " · error".to_string(), Some(theme.error))
            }
            TaskStatus::Pending | TaskStatus::Running => {
                let tail = if session.has_review_progress {
                    format!(
                        " · {}",
                        format_review_stage_counts(
                            session.review_stage,
                            session.bugs_found,
                            session.bugs_verified,
                            session.bugs_refuted
                        )
                    )
                } else {
                    " · setting up".to_string()
                };
                (DIAMOND_OPEN, tail, None)
            }
        };
        return element! { View(flex_direction: FlexDirection::Row) {
            Text(content: format!("{diamond} "), color: theme.background)
            #(rainbow)
            Text(content: tail, color: tail_color, dim: true)
        }}
        .into_any();
    }
    match session.status {
        TaskStatus::Completed => element! { Text(content: "done".to_string(), weight: Weight::Bold, color: theme.success, dim: true) }.into_any(),
        TaskStatus::Failed | TaskStatus::Killed => element! { Text(content: "error".to_string(), weight: Weight::Bold, color: theme.error, dim: true) }.into_any(),
        TaskStatus::Pending | TaskStatus::Running if session.todo_total == 0 => element! { Text(content: format!("{}…", session.status_label.as_deref().unwrap_or("running")), dim: true) }.into_any(),
        TaskStatus::Pending | TaskStatus::Running => element! { Text(content: format!("{}/{}", session.todo_completed, session.todo_total), dim: true) }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn review_counts_match_stage_vocabulary_and_refuted_gate() {
        assert_eq!(
            format_review_stage_counts(None, 3, 1, 9),
            "3 found · 1 verified"
        );
        assert_eq!(
            format_review_stage_counts(Some(ReviewStage::Finding), 0, 0, 0),
            "finding"
        );
        assert_eq!(
            format_review_stage_counts(Some(ReviewStage::Verifying), 3, 1, 0),
            "3 found · 1 verified"
        );
        assert_eq!(
            format_review_stage_counts(Some(ReviewStage::Synthesizing), 3, 2, 1),
            "2 verified · 1 refuted · deduping"
        );
    }
    #[test]
    fn ordinary_and_review_terminal_copy_render() {
        let theme = *crate::utils::theme::current();
        // The component reads `settings.prefers_reduced_motion` (it gates the
        // animation frame). Defaults are the fixture: both assertions are on
        // completed-state copy, which is static either way.
        let ordinary = element! { ContextProvider(value: Context::owned(theme)) { crate::state::app_state::AppStateProvider(children: crate::state::app_state::ProviderChildren::new(|| element! { RemoteSessionProgress(session: Some(RemoteSessionProgressData { status: TaskStatus::Completed, ..Default::default() })) }.into_any())) } }.render(Some(60)).to_string();
        assert!(ordinary.contains("done"));
        let review = element! { ContextProvider(value: Context::owned(theme)) { crate::state::app_state::AppStateProvider(children: crate::state::app_state::ProviderChildren::new(|| element! { RemoteSessionProgress(session: Some(RemoteSessionProgressData { status: TaskStatus::Completed, is_remote_review: true, ..Default::default() })) }.into_any())) } }.render(Some(80)).to_string();
        assert!(review.contains("ultrareview ready · shift+↓ to view"));
    }
}
