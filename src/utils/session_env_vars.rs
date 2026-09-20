//! Session-scoped child-process environment variables.
//!
//! Maps to: CC `utils/sessionEnvVars.ts:1-24`.

use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

static SESSION_ENV_VARS: LazyLock<RwLock<BTreeMap<String, String>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

/// Maps to CC `getSessionEnvVars()`.
pub fn get_session_env_vars() -> BTreeMap<String, String> {
    SESSION_ENV_VARS
        .read()
        .map(|vars| vars.clone())
        .unwrap_or_default()
}

/// Maps to CC `setSessionEnvVar(name, value)`.
pub fn set_session_env_var(name: impl Into<String>, value: impl Into<String>) {
    if let Ok(mut vars) = SESSION_ENV_VARS.write() {
        vars.insert(name.into(), value.into());
    }
}

/// Maps to CC `deleteSessionEnvVar(name)`.
pub fn delete_session_env_var(name: &str) {
    if let Ok(mut vars) = SESSION_ENV_VARS.write() {
        vars.remove(name);
    }
}

/// Maps to CC `clearSessionEnvVars()`.
pub fn clear_session_env_vars() {
    if let Ok(mut vars) = SESSION_ENV_VARS.write() {
        vars.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_env_var_mutations_match_official_map_contract() {
        struct Restore(BTreeMap<String, String>);
        impl Drop for Restore {
            fn drop(&mut self) {
                if let Ok(mut vars) = SESSION_ENV_VARS.write() {
                    *vars = std::mem::take(&mut self.0);
                }
            }
        }

        let _lock = crate::utils::env_utils::TEST_ENV_LOCK.lock().unwrap();
        let _restore = Restore(get_session_env_vars());
        clear_session_env_vars();
        set_session_env_var("A", "one");
        set_session_env_var("B", "two");
        assert_eq!(
            get_session_env_vars().get("A").map(String::as_str),
            Some("one")
        );
        delete_session_env_var("A");
        assert!(!get_session_env_vars().contains_key("A"));
        clear_session_env_vars();
    }
}
