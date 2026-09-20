//! Main-screen-safe subset of official `WebSearchTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

use super::{Output, WebSearchHit, WebSearchResultItem, WebSearchSearchResult};

/// Maps to: CC `tools/WebSearchTool/UI.tsx:120-127` `getToolUseSummary`.
pub fn get_tool_use_summary(input: Option<&serde_json::Value>) -> Option<String> {
    let query = input?
        .get("query")?
        .as_str()
        .filter(|query| !query.is_empty())?;
    Some(crate::utils::truncate::truncate_to_width(
        query,
        crate::constants::tool_limits::TOOL_SUMMARY_MAX_LENGTH,
    ))
}

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `query`, `results`, and
/// `durationSeconds` are required, and each `results` element must satisfy
/// the `searchResult | string` union (`WebSearchTool.ts:56-67`).
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    let query = map.get("query")?.as_str()?.to_string();
    let duration_seconds = map.get("durationSeconds")?.as_f64()?;
    let results = map
        .get("results")?
        .as_array()?
        .iter()
        .map(|item| match item {
            serde_json::Value::String(text) => Some(WebSearchResultItem::Text(text.clone())),
            serde_json::Value::Object(result) => {
                Some(WebSearchResultItem::SearchResult(WebSearchSearchResult {
                    tool_use_id: result.get("tool_use_id")?.as_str()?.to_string(),
                    content: result
                        .get("content")?
                        .as_array()?
                        .iter()
                        .map(|hit| {
                            Some(WebSearchHit {
                                title: hit.get("title")?.as_str()?.to_string(),
                                url: hit.get("url")?.as_str()?.to_string(),
                            })
                        })
                        .collect::<Option<Vec<_>>>()?,
                }))
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Output {
        query,
        results,
        duration_seconds,
    })
}

/// Serializes [`Output`] back to CC's exact `toolUseResult` wire shape.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    serde_json::json!({
        "query": output.query,
        "results": output
            .results
            .iter()
            .map(|item| match item {
                WebSearchResultItem::Text(text) => serde_json::Value::String(text.clone()),
                WebSearchResultItem::SearchResult(result) => serde_json::json!({
                    "tool_use_id": result.tool_use_id,
                    "content": result
                        .content
                        .iter()
                        .map(|hit| serde_json::json!({"title": hit.title, "url": hit.url}))
                        .collect::<Vec<_>>(),
                }),
            })
            .collect::<Vec<_>>(),
        "durationSeconds": output.duration_seconds,
    })
}

/// Maps to: CC `WebSearchTool/UI.tsx:101-118` `renderToolResultMessage` as
/// invoked by `UserToolSuccessMessage.tsx:80-96` — parse the raw
/// `toolUseResult` with the tool's own output schema and render the
/// "Did N search(es) in Xs" chrome; render nothing when it does not parse.
///
/// This is the by-tool-name entry the dispatch calls — the only WebSearch
/// render path now that the WebSearch display variant is gone.
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
    _status: crate::types::message::ToolResultStatus,
    _fallback: &str,
    _options: &crate::components::messages::user_tool_result_message::utils::ToolRenderOptions,
) -> Vec<ToolRenderLine> {
    // CC bails on a missing `toolUseResult` before touching the tool
    // (`UserToolSuccessMessage.tsx:72`).
    let Some(raw_output) = raw_output else {
        return Vec::new();
    };
    // CC: `safeParse` failure returns null, i.e. the row renders nothing
    // (`UserToolSuccessMessage.tsx:81`).
    let Some(output) = parse_output(raw_output) else {
        return Vec::new();
    };
    // CC `getSearchSummary` (UI.tsx:13-30): non-string entries count.
    let search_count = output
        .results
        .iter()
        .filter(|item| matches!(item, WebSearchResultItem::SearchResult(_)))
        .count();
    let time_display = web_search_time_display(output.duration_seconds);
    vec![ToolRenderLine::new(
        format!(
            "Did {search_count} {} in {time_display}",
            if search_count == 1 {
                "search"
            } else {
                "searches"
            }
        ),
        ToolRenderTone::Normal,
    )]
}

/// CC `UI.tsx:103-106` timeDisplay.
fn web_search_time_display(duration_seconds: f64) -> String {
    if duration_seconds >= 1.0 {
        format!("{}s", duration_seconds.round() as i64)
    } else {
        format!("{}ms", (duration_seconds * 1000.0).round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Maps to: CC `WebSearchTool/UI.tsx:101-118` — the raw entry is now the
    /// only WebSearch render path; string entries never count as searches.
    #[test]
    fn web_search_summary_pluralizes_like_official_copy() {
        let render = |raw: serde_json::Value| {
            render_tool_result_message(
                Some(&raw),
                crate::types::message::ToolResultStatus::Success,
                "",
                &Default::default(),
            )
        };
        let one = render(serde_json::json!({
            "query": "rust",
            "results": [
                "commentary",
                {"tool_use_id": "srvtoolu_1", "content": [{"title": "T", "url": "https://a"}]}
            ],
            "durationSeconds": 0.9
        }));
        assert_eq!(one[0].text, "Did 1 search in 900ms");
        assert_eq!(one[0].tone, ToolRenderTone::Normal);

        let many = render(serde_json::json!({
            "query": "rust",
            "results": [
                {"tool_use_id": "srvtoolu_1", "content": []},
                {"tool_use_id": "srvtoolu_2", "content": []}
            ],
            "durationSeconds": 3.2
        }));
        assert_eq!(many[0].text, "Did 2 searches in 3s");

        assert!(render(serde_json::json!({"query": "rust"})).is_empty());
    }
}
