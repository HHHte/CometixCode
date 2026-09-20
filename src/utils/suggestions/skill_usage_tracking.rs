//! Maps to: CC `utils/suggestions/skillUsageTracking.ts`.
//!
//! Usage is a ranking hint for prompt commands. The persisted owner remains
//! `utils/config.rs`; this module only applies the one-minute process debounce
//! and the seven-day exponential decay used by command suggestions.

use crate::utils::config::{GlobalConfig, load_global_config, save_global_config};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEBOUNCE: Duration = Duration::from_secs(60);
const WEEK_SECONDS: f64 = 7.0 * 24.0 * 60.0 * 60.0;

static LAST_WRITE_BY_SKILL: LazyLock<Mutex<HashMap<String, SystemTime>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Maps to CC `recordSkillUsage(skillName)`.
pub fn record_skill_usage(skill_name: &str) {
    if skill_name.is_empty() {
        return;
    }
    let now = SystemTime::now();
    if let Ok(mut writes) = LAST_WRITE_BY_SKILL.lock() {
        if writes
            .get(skill_name)
            .and_then(|last| now.duration_since(*last).ok())
            .is_some_and(|elapsed| elapsed < DEBOUNCE)
        {
            return;
        }
        writes.insert(skill_name.to_owned(), now);
    }

    let timestamp = now
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as i64);
    if let Err(error) = save_global_config(|config| {
        let usage = config.skill_usage.get_or_insert_with(HashMap::new);
        let entry = usage
            .entry(skill_name.to_owned())
            .or_insert_with(|| serde_json::json!({}));
        if let Some(object) = entry.as_object_mut() {
            let count = object
                .get("usageCount")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
                .saturating_add(1);
            object.insert("usageCount".to_owned(), serde_json::json!(count));
            object.insert("lastUsedAt".to_owned(), serde_json::json!(timestamp));
        }
    }) {
        tracing::debug!(%error, skill = skill_name, "failed to record skill usage");
    }
}

/// Maps to CC `getSkillUsageScore(skillName)`.
pub fn get_skill_usage_score(skill_name: &str) -> f64 {
    let config: GlobalConfig = load_global_config();
    let Some(usage) = config
        .skill_usage
        .as_ref()
        .and_then(|entries| entries.get(skill_name))
    else {
        return 0.0;
    };
    let count = usage
        .get("usageCount")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as f64;
    let last_used = usage
        .get("lastUsedAt")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    if count <= 0.0 || last_used <= 0 {
        return 0.0;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as i64);
    let elapsed_seconds = (now.saturating_sub(last_used) as f64 / 1000.0).max(0.0);
    let recency = (0.5_f64).powf(elapsed_seconds / WEEK_SECONDS).max(0.1);
    count * recency
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_score_applies_weekly_decay_and_floor() {
        let previous = crate::utils::config::replace_test_global_config(Some(GlobalConfig {
            skill_usage: Some(HashMap::from([
                (
                    "recent".to_owned(),
                    serde_json::json!({"usageCount": 4, "lastUsedAt": 1_000_000_000_000i64}),
                ),
                (
                    "old".to_owned(),
                    serde_json::json!({"usageCount": 1, "lastUsedAt": 1i64}),
                ),
            ])),
            ..GlobalConfig::default()
        }));
        let recent = get_skill_usage_score("recent");
        let old = get_skill_usage_score("old");
        assert!(recent > 0.0);
        assert!((0.1..=1.0).contains(&old));
        assert_eq!(get_skill_usage_score("missing"), 0.0);
        crate::utils::config::replace_test_global_config(previous);
    }
}
