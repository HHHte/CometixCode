//! MCP elicitation service boundary.
//! Maps to: CC `services/mcp/elicitationHandler.ts`.
//!
//! This module owns the data model and pure state transitions for MCP
//! elicitation requests/completion notifications. The terminal UI lives in
//! `components/mcp/elicitation_dialog.rs`, and the rmcp transport integration
//! remains in `services/mcp/client.rs`.

use crate::services::hooks::{HookBlockingError, HookElicitationResponse, HookResult};
use serde_json::Value;

/// Maps to: CC `ElicitationWaitingState`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElicitationWaitingState {
    pub action_label: String,
    pub show_cancel: bool,
}

/// Maps to: CC `onWaitingDismiss` action union.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElicitationWaitingDismissAction {
    Dismiss,
    Retry,
    Cancel,
}

/// Maps to: CC `ElicitRequestParams` mode split.
#[derive(Clone, Debug, PartialEq)]
pub enum ElicitationRequestParams {
    Form {
        message: String,
        requested_schema: Value,
    },
    Url {
        message: String,
        url: String,
        elicitation_id: Option<String>,
    },
}

/// Maps to: CC `getElicitationMode(...)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElicitationMode {
    Form,
    Url,
}

impl ElicitationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Form => "form",
            Self::Url => "url",
        }
    }
}

pub fn get_elicitation_mode(params: &ElicitationRequestParams) -> ElicitationMode {
    match params {
        ElicitationRequestParams::Form { .. } => ElicitationMode::Form,
        ElicitationRequestParams::Url { .. } => ElicitationMode::Url,
    }
}

/// Maps to: CC `ElicitResult['action']`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElicitationAction {
    Accept,
    Decline,
    Cancel,
}

impl ElicitationAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Decline => "decline",
            Self::Cancel => "cancel",
        }
    }
}

impl From<&str> for ElicitationAction {
    fn from(value: &str) -> Self {
        match value {
            "accept" => Self::Accept,
            "decline" => Self::Decline,
            _ => Self::Cancel,
        }
    }
}

/// Maps to: CC `ElicitResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct ElicitationResult {
    pub action: ElicitationAction,
    pub content: Option<Value>,
}

impl ElicitationResult {
    pub fn new(action: ElicitationAction) -> Self {
        Self {
            action,
            content: None,
        }
    }
}

/// Maps to: CC `ElicitationRequestEvent` minus the JS callback/signal fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ElicitationRequestEvent {
    pub server_name: String,
    pub request_id: String,
    pub params: ElicitationRequestParams,
    pub waiting_state: Option<ElicitationWaitingState>,
    pub completed: bool,
}

impl ElicitationRequestEvent {
    pub fn new(
        server_name: impl Into<String>,
        request_id: impl Into<String>,
        params: ElicitationRequestParams,
    ) -> Self {
        let waiting_state = match &params {
            ElicitationRequestParams::Url {
                elicitation_id: Some(_),
                ..
            } => Some(ElicitationWaitingState {
                action_label: "Skip confirmation".to_string(),
                show_cancel: false,
            }),
            _ => None,
        };
        Self {
            server_name: server_name.into(),
            request_id: request_id.into(),
            params,
            waiting_state,
            completed: false,
        }
    }

    /// Maps to: CC `callMCPToolWithUrlElicitationRetry(...)` queued
    /// `waitingState: { actionLabel: 'Retry now', showCancel: true }`.
    pub fn with_error_retry_waiting_state(mut self) -> Self {
        self.waiting_state = Some(ElicitationWaitingState {
            action_label: "Retry now".to_string(),
            show_cancel: true,
        });
        self
    }
}

/// Maps to: CC `onWaitingDismiss` in `callMCPToolWithUrlElicitationRetry(...)`.
pub fn url_retry_waiting_dismiss_result(
    action: ElicitationWaitingDismissAction,
) -> ElicitationResult {
    match action {
        ElicitationWaitingDismissAction::Retry => ElicitationResult::new(ElicitationAction::Accept),
        ElicitationWaitingDismissAction::Dismiss | ElicitationWaitingDismissAction::Cancel => {
            ElicitationResult::new(ElicitationAction::Cancel)
        }
    }
}

/// Maps to: CC URL elicitation declined/canceled result text in
/// `callMCPToolWithUrlElicitationRetry(...)`.
pub fn url_elicitation_required_result_message(
    action: ElicitationAction,
    source: &str,
    tool: &str,
) -> String {
    let verb = match action {
        ElicitationAction::Decline => "declined",
        ElicitationAction::Cancel => "canceled",
        ElicitationAction::Accept => "accepted",
    };
    format!(
        "URL elicitation was {verb} by {source}. The tool \"{tool}\" could not complete because it requires the user to open a URL."
    )
}

/// Maps to: CC `findElicitationInQueue(...)`.
pub fn find_elicitation_in_queue(
    queue: &[ElicitationRequestEvent],
    server_name: &str,
    elicitation_id: &str,
) -> Option<usize> {
    queue.iter().position(|event| {
        event.server_name == server_name
            && matches!(
                &event.params,
                ElicitationRequestParams::Url {
                    elicitation_id: Some(id),
                    ..
                } if id == elicitation_id
            )
    })
}

/// Maps to: CC completion-notification `setAppState(...)` queue update.
pub fn mark_elicitation_complete(
    queue: &[ElicitationRequestEvent],
    server_name: &str,
    elicitation_id: &str,
) -> (Vec<ElicitationRequestEvent>, bool) {
    let Some(index) = find_elicitation_in_queue(queue, server_name, elicitation_id) else {
        return (queue.to_vec(), false);
    };
    let mut next = queue.to_vec();
    if let Some(event) = next.get_mut(index) {
        event.completed = true;
    }
    (next, true)
}

fn hook_response_to_result(response: &HookElicitationResponse) -> ElicitationResult {
    ElicitationResult {
        action: ElicitationAction::from(response.action.as_str()),
        content: response.content.clone(),
    }
}

/// Maps to: CC `runElicitationHooks(...)` result reduction.
pub fn reduce_elicitation_hook_results(
    results: &[HookResult],
) -> (Option<ElicitationResult>, Option<HookBlockingError>) {
    let mut response = None;
    let mut blocking_error = None;
    for result in results {
        if let Some(error) = &result.blocking_error {
            blocking_error = Some(error.clone());
        }
        if let Some(hook_response) = &result.elicitation_response {
            response = Some(hook_response_to_result(hook_response));
        }
    }
    (response, blocking_error)
}

/// Maps to: CC `runElicitationResultHooks(...)` final-result reduction.
pub fn reduce_elicitation_result_hook_results(
    original: &ElicitationResult,
    results: &[HookResult],
) -> (ElicitationResult, Option<HookBlockingError>) {
    let mut final_result = original.clone();
    let mut blocking_error = None;
    for result in results {
        if let Some(error) = &result.blocking_error {
            blocking_error = Some(error.clone());
        }
        if let Some(hook_response) = &result.elicitation_result_response {
            final_result = hook_response_to_result(hook_response);
            if final_result.content.is_none() {
                final_result.content = original.content.clone();
            }
        }
    }
    if blocking_error.is_some() {
        final_result = ElicitationResult::new(ElicitationAction::Decline);
    }
    (final_result, blocking_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::hooks::{HookBlockingError, HookElicitationResponse, HookResult};

    #[test]
    fn find_and_mark_elicitation_completion_matches_official_queue_lookup() {
        let form = ElicitationRequestEvent::new(
            "docs",
            "1",
            ElicitationRequestParams::Form {
                message: "form".to_string(),
                requested_schema: serde_json::json!({"type":"object"}),
            },
        );
        let url = ElicitationRequestEvent::new(
            "docs",
            "2",
            ElicitationRequestParams::Url {
                message: "open".to_string(),
                url: "https://example.com/auth".to_string(),
                elicitation_id: Some("elicit-1".to_string()),
            },
        );
        let queue = vec![form, url];

        assert_eq!(
            find_elicitation_in_queue(&queue, "docs", "elicit-1"),
            Some(1)
        );
        assert_eq!(find_elicitation_in_queue(&queue, "other", "elicit-1"), None);

        let (updated, found) = mark_elicitation_complete(&queue, "docs", "elicit-1");
        assert!(found);
        assert!(!queue[1].completed);
        assert!(updated[1].completed);
    }

    #[test]
    fn url_event_with_elicitation_id_gets_official_waiting_state_default() {
        let event = ElicitationRequestEvent::new(
            "docs",
            "2",
            ElicitationRequestParams::Url {
                message: "open".to_string(),
                url: "https://example.com/auth".to_string(),
                elicitation_id: Some("elicit-1".to_string()),
            },
        );
        assert_eq!(get_elicitation_mode(&event.params), ElicitationMode::Url);
        assert_eq!(
            event.waiting_state,
            Some(ElicitationWaitingState {
                action_label: "Skip confirmation".to_string(),
                show_cancel: false,
            })
        );
    }

    #[test]
    fn url_retry_waiting_dismiss_result_matches_official_retry_flow() {
        assert_eq!(
            url_retry_waiting_dismiss_result(ElicitationWaitingDismissAction::Retry),
            ElicitationResult::new(ElicitationAction::Accept)
        );
        assert_eq!(
            url_retry_waiting_dismiss_result(ElicitationWaitingDismissAction::Cancel),
            ElicitationResult::new(ElicitationAction::Cancel)
        );
        assert_eq!(
            url_elicitation_required_result_message(
                ElicitationAction::Decline,
                "the user",
                "login"
            ),
            "URL elicitation was declined by the user. The tool \"login\" could not complete because it requires the user to open a URL."
        );
    }

    #[test]
    fn reduce_elicitation_hook_results_keeps_last_response_and_blocking_error() {
        let mut first = HookResult::default();
        first.elicitation_response = Some(HookElicitationResponse {
            action: "accept".to_string(),
            content: Some(serde_json::json!({"a": 1})),
        });
        let mut second = HookResult::default();
        second.elicitation_response = Some(HookElicitationResponse {
            action: "decline".to_string(),
            content: None,
        });
        second.blocking_error = Some(HookBlockingError {
            blocking_error: "blocked".to_string(),
            command: "hook".to_string(),
        });

        let (response, blocking) = reduce_elicitation_hook_results(&[first, second]);
        assert_eq!(
            response,
            Some(ElicitationResult::new(ElicitationAction::Decline))
        );
        assert_eq!(blocking.unwrap().blocking_error, "blocked");
    }

    #[test]
    fn reduce_elicitation_result_hook_results_preserves_original_content_when_override_omits_it() {
        let original = ElicitationResult {
            action: ElicitationAction::Accept,
            content: Some(serde_json::json!({"email":"user@example.com"})),
        };
        let mut hook = HookResult::default();
        hook.elicitation_result_response = Some(HookElicitationResponse {
            action: "accept".to_string(),
            content: None,
        });

        let (result, blocking) = reduce_elicitation_result_hook_results(&original, &[hook]);
        assert!(blocking.is_none());
        assert_eq!(result, original);
    }

    #[test]
    fn reduce_elicitation_result_hook_results_declines_when_blocking() {
        let original = ElicitationResult {
            action: ElicitationAction::Accept,
            content: Some(serde_json::json!({"email":"user@example.com"})),
        };
        let mut hook = HookResult::default();
        hook.blocking_error = Some(HookBlockingError {
            blocking_error: "no".to_string(),
            command: "hook".to_string(),
        });

        let (result, blocking) = reduce_elicitation_result_hook_results(&original, &[hook]);
        assert!(blocking.is_some());
        assert_eq!(result, ElicitationResult::new(ElicitationAction::Decline));
    }
}
