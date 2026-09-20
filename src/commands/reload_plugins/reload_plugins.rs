//! Maps to: CC `commands/reload-plugins/reload-plugins.ts`.

/// Maps to: CC `reload-plugins.ts:10-62#call`.
/// L1: the sole `{type:'text', value}` return variant is the returned String;
/// the processSlashCommand owner constructs the transcript rows.
pub async fn call(store: &crate::state::store::AppStore) -> anyhow::Result<String> {
    // Partial: CC :24-38 gates redownloadUserSettings/notifyChange behind
    // DOWNLOAD_USER_SETTINGS + remote mode. That build feature and the
    // services/settingsSync owner are not ported. This local-only path does
    // not claim to re-pull remote user settings or issue a replacement request.

    #[cfg(not(test))]
    let r = crate::utils::plugins::refresh::refresh_active_plugins(store).await?;
    #[cfg(test)]
    let r = if let Some(result) = TEST_REFRESH_RESULT.with(|slot| slot.borrow_mut().take()) {
        result?
    } else {
        crate::utils::plugins::refresh::refresh_active_plugins(store).await?
    };

    let parts = [
        n(r.enabled_count, "plugin"),
        n(r.command_count, "skill"),
        n(r.agent_count, "agent"),
        n(r.hook_count, "hook"),
        n(r.mcp_count, "plugin MCP server"),
        n(r.lsp_count, "plugin LSP server"),
    ];
    let mut msg = format!("Reloaded: {}", parts.join(" · "));
    if r.error_count > 0 {
        msg.push_str(&format!(
            "\n{} during load. Run /doctor for details.",
            n(r.error_count, "error")
        ));
    }
    Ok(msg)
}

/// Maps to: CC `reload-plugins.ts:65-67#n`.
fn n(count: usize, noun: &str) -> String {
    format!(
        "{count} {}",
        crate::utils::string_utils::plural(count, noun, None)
    )
}

#[cfg(test)]
thread_local! {
    // Test-only imported refresh dependency. The command body and output
    // branches remain production code; normal tests without a fixture use
    // the real refresh implementation.
    static TEST_REFRESH_RESULT: std::cell::RefCell<Option<anyhow::Result<crate::utils::plugins::refresh::RefreshActivePluginsResult>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_plugins_call_matches_official_bun_count_and_error_output() {
        use crate::utils::plugins::refresh::RefreshActivePluginsResult;
        for (result, expected) in [
            (
                RefreshActivePluginsResult::default(),
                "Reloaded: 0 plugins · 0 skills · 0 agents · 0 hooks · 0 plugin MCP servers · 0 plugin LSP servers",
            ),
            (
                RefreshActivePluginsResult {
                    enabled_count: 1,
                    command_count: 1,
                    agent_count: 1,
                    hook_count: 1,
                    mcp_count: 1,
                    lsp_count: 1,
                    error_count: 1,
                    ..Default::default()
                },
                "Reloaded: 1 plugin · 1 skill · 1 agent · 1 hook · 1 plugin MCP server · 1 plugin LSP server\n1 error during load. Run /doctor for details.",
            ),
            (
                RefreshActivePluginsResult {
                    enabled_count: 2,
                    command_count: 3,
                    agent_count: 4,
                    hook_count: 5,
                    mcp_count: 6,
                    lsp_count: 7,
                    error_count: 8,
                    ..Default::default()
                },
                "Reloaded: 2 plugins · 3 skills · 4 agents · 5 hooks · 6 plugin MCP servers · 7 plugin LSP servers\n8 errors during load. Run /doctor for details.",
            ),
        ] {
            TEST_REFRESH_RESULT.with(|slot| *slot.borrow_mut() = Some(Ok(result)));
            let store = crate::state::store::AppStore::new(Default::default(), None);
            let actual = futures::executor::block_on(call(&store)).unwrap();
            assert_eq!(actual, expected);
        }
        TEST_REFRESH_RESULT
            .with(|slot| *slot.borrow_mut() = Some(Err(anyhow::anyhow!("load failed"))));
        let store = crate::state::store::AppStore::new(Default::default(), None);
        let error = futures::executor::block_on(call(&store)).unwrap_err();
        assert_eq!(error.to_string(), "load failed");
    }
}
