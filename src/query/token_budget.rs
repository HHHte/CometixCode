//! Token budget continuation decisions.
//! Maps to official `query/tokenBudget.ts`.

use crate::utils::token_budget::get_budget_continuation_message;

const COMPLETION_THRESHOLD: f64 = 0.9;
const DIMINISHING_THRESHOLD: i64 = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetTracker {
    pub continuation_count: u32,
    pub last_delta_tokens: i64,
    pub last_global_turn_tokens: i64,
    pub started_at_ms: i64,
}

pub fn create_budget_tracker() -> BudgetTracker {
    BudgetTracker {
        continuation_count: 0,
        last_delta_tokens: 0,
        last_global_turn_tokens: 0,
        started_at_ms: now_ms(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenBudgetDecision {
    Continue(ContinueDecision),
    Stop(StopDecision),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinueDecision {
    pub nudge_message: String,
    pub continuation_count: u32,
    pub pct: i64,
    pub turn_tokens: i64,
    pub budget: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopDecision {
    pub completion_event: Option<CompletionEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEvent {
    pub continuation_count: u32,
    pub pct: i64,
    pub turn_tokens: i64,
    pub budget: i64,
    pub diminishing_returns: bool,
    pub duration_ms: i64,
}

/// Maps to official `checkTokenBudget(...)`.
pub fn check_token_budget(
    tracker: &mut BudgetTracker,
    agent_id: Option<&str>,
    budget: Option<i64>,
    global_turn_tokens: i64,
) -> TokenBudgetDecision {
    let Some(budget) = budget else {
        return stop(None);
    };
    if agent_id.is_some() || budget <= 0 {
        return stop(None);
    }

    let turn_tokens = global_turn_tokens;
    let pct = ((turn_tokens as f64 / budget as f64) * 100.0).round() as i64;
    let delta_since_last_check = global_turn_tokens - tracker.last_global_turn_tokens;

    let is_diminishing = tracker.continuation_count >= 3
        && delta_since_last_check < DIMINISHING_THRESHOLD
        && tracker.last_delta_tokens < DIMINISHING_THRESHOLD;

    if !is_diminishing && (turn_tokens as f64) < (budget as f64 * COMPLETION_THRESHOLD) {
        tracker.continuation_count += 1;
        tracker.last_delta_tokens = delta_since_last_check;
        tracker.last_global_turn_tokens = global_turn_tokens;
        return TokenBudgetDecision::Continue(ContinueDecision {
            nudge_message: get_budget_continuation_message(pct, turn_tokens, budget),
            continuation_count: tracker.continuation_count,
            pct,
            turn_tokens,
            budget,
        });
    }

    if is_diminishing || tracker.continuation_count > 0 {
        return stop(Some(CompletionEvent {
            continuation_count: tracker.continuation_count,
            pct,
            turn_tokens,
            budget,
            diminishing_returns: is_diminishing,
            duration_ms: now_ms() - tracker.started_at_ms,
        }));
    }

    stop(None)
}

fn stop(completion_event: Option<CompletionEvent>) -> TokenBudgetDecision {
    TokenBudgetDecision::Stop(StopDecision { completion_event })
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_token_budget_continues_under_threshold_like_official() {
        let mut tracker = BudgetTracker {
            continuation_count: 0,
            last_delta_tokens: 0,
            last_global_turn_tokens: 0,
            started_at_ms: now_ms(),
        };

        let decision = check_token_budget(&mut tracker, None, Some(1_000), 500);

        assert!(matches!(
            decision,
            TokenBudgetDecision::Continue(ContinueDecision {
                continuation_count: 1,
                pct: 50,
                turn_tokens: 500,
                budget: 1_000,
                ..
            })
        ));
        assert_eq!(tracker.last_delta_tokens, 500);
        assert_eq!(tracker.last_global_turn_tokens, 500);
    }

    #[test]
    fn check_token_budget_stops_with_completion_event_after_continuation() {
        let mut tracker = BudgetTracker {
            continuation_count: 1,
            last_delta_tokens: 600,
            last_global_turn_tokens: 600,
            started_at_ms: now_ms(),
        };

        let decision = check_token_budget(&mut tracker, None, Some(1_000), 950);

        assert!(matches!(
            decision,
            TokenBudgetDecision::Stop(StopDecision {
                completion_event: Some(CompletionEvent {
                    continuation_count: 1,
                    pct: 95,
                    turn_tokens: 950,
                    budget: 1_000,
                    diminishing_returns: false,
                    ..
                })
            })
        ));
    }

    #[test]
    fn check_token_budget_detects_diminishing_returns_like_official() {
        let mut tracker = BudgetTracker {
            continuation_count: 3,
            last_delta_tokens: 100,
            last_global_turn_tokens: 900,
            started_at_ms: now_ms(),
        };

        let decision = check_token_budget(&mut tracker, None, Some(10_000), 1_000);

        assert!(matches!(
            decision,
            TokenBudgetDecision::Stop(StopDecision {
                completion_event: Some(CompletionEvent {
                    diminishing_returns: true,
                    ..
                })
            })
        ));
    }
}
