//! Maps to: CC `hooks/unifiedSuggestions.ts`.
//!
//! The source hook deliberately owns the merge between file paths, MCP
//! resources and agent definitions.  The Rust producers remain in their
//! original owners (`file_suggestions.rs` and `load_agents_dir.rs`); this
//! module only adapts their rows into the one ranked result set consumed by
//! `use_typeahead`.

use crate::components::prompt_input::prompt_input_footer_suggestions::SuggestionItem;
use crate::services::mcp::types::ServerResource;
use crate::tools::agent_tool::agent_color_manager::get_agent_color;
use crate::tools::agent_tool::load_agents_dir::AgentDefinition;
use crate::utils::fuse::{BitapSearch, FuseKey, FuseOptions, KeyStore};
use crate::utils::truncate::truncate_to_width;
use std::collections::BTreeMap;

const MAX_UNIFIED_SUGGESTIONS: usize = 15;
const DESCRIPTION_MAX_WIDTH: usize = 60;

#[derive(Clone, Debug)]
struct ScoredSuggestion {
    item: SuggestionItem,
    /// CC's cross-source currency: 0..1, **lower is better** (:196
    /// `sort((a, b) => a.score - b.score)`). File rows carry the rank from
    /// `FileIndex.search`; the other sources carry a Fuse.js score, which
    /// lives on the same 0..1 scale with `threshold: 0.6` (:176).
    score: f64,
    source_order: usize,
}

/// Fuse.js `threshold` from the source config (:176). Nothing scoring worse
/// than this would have been returned by `fuse.search` at all.
/// Maps to the source's Fuse construction (:174-184): `threshold: 0.6` and no
/// `ignoreLocation`, so `location: 0` / `distance: 100` stay at their defaults.
///
/// The `keys` table is all five the source declares (:177-183) — displayText 2,
/// name 3, server 1, description 1, agentType 3, totalling 10. Fuse normalizes
/// that table once, so `name` is `3/10` on an MCP row (which carries four of
/// the five) and `agentType` is `3/10` on an agent row (three of the five).
/// Handing `KeyStore` only the keys a given row populates would make them
/// `3/7` and `3/6` instead, ranking the two shapes against different scales.
fn fuse_options() -> FuseOptions {
    FuseOptions {
        threshold: 0.6,
        keys: KeyStore::new(&[2.0, 3.0, 1.0, 1.0, 3.0]),
        ..FuseOptions::default()
    }
}

fn description(value: &str) -> String {
    truncate_to_width(value, DESCRIPTION_MAX_WIDTH)
}

fn file_item(item: SuggestionItem, score: f64, source_order: usize) -> ScoredSuggestion {
    let mut metadata = item.metadata.unwrap_or_else(|| serde_json::json!({}));
    if !metadata.is_object() {
        metadata = serde_json::json!({});
    }
    if let Some(object) = metadata.as_object_mut() {
        object.insert("type".to_string(), serde_json::json!("file"));
        object.insert("score".to_string(), serde_json::json!(score));
    }
    ScoredSuggestion {
        item: SuggestionItem {
            metadata: Some(metadata),
            ..item
        },
        score,
        source_order,
    }
}

fn mcp_item(resource: &ServerResource, source_order: usize) -> SuggestionItem {
    let display = format!("{}:{}", resource.server, resource.uri);
    let description_text = resource
        .description
        .as_deref()
        .filter(|text| !text.is_empty())
        .or_else(|| (!resource.name.is_empty()).then_some(resource.name.as_str()))
        .unwrap_or(resource.uri.as_str());
    SuggestionItem {
        id: format!("mcp-resource-{}__{}", resource.server, resource.uri),
        display_text: display,
        tag: None,
        command_text: resource.uri.clone(),
        description: description(description_text),
        metadata: Some(serde_json::json!({
            "type": "mcp_resource",
            "server": resource.server,
            "uri": resource.uri,
            "sourceOrder": source_order,
        })),
        color: None,
    }
}

fn agent_item(agent: &AgentDefinition, source_order: usize) -> SuggestionItem {
    let color = get_agent_color(&agent.agent_type);
    SuggestionItem {
        id: format!("agent-{}", agent.agent_type),
        display_text: format!("{} (agent)", agent.agent_type),
        tag: None,
        command_text: agent.agent_type.clone(),
        description: description(&agent.when_to_use),
        metadata: Some(serde_json::json!({
            "type": "agent",
            "agentType": agent.agent_type,
            "color": color.map(|key| format!("{key:?}")),
            "sourceOrder": source_order,
        })),
        color,
    }
}

/// Maps to CC `generateAgentSuggestions` (`unifiedSuggestions.ts:77-108`).
/// Agent discovery uses a case-insensitive substring filter before the unified
/// Fuse pass; a fuzzy-only match must not make an agent row appear.
fn agent_matches_query(agent: &AgentDefinition, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query = query.to_lowercase();
    agent.agent_type.to_lowercase().contains(&query)
        || format!("{} (agent)", agent.agent_type)
            .to_lowercase()
            .contains(&query)
}

/// Maps to CC `generateUnifiedSuggestions(query, mcpResources, agents,
/// showOnEmpty)`.  File discovery remains asynchronous in its source owner;
/// this function is async only so the typeahead worker can await that exact
/// producer before ranking the merged rows.
pub async fn generate_unified_suggestions(
    query: &str,
    mcp_resources: &BTreeMap<String, Vec<ServerResource>>,
    agents: &[AgentDefinition],
    show_on_empty: bool,
) -> Vec<SuggestionItem> {
    if query.is_empty() && !show_on_empty {
        return Vec::new();
    }

    let file_suggestions =
        crate::hooks::file_suggestions::generate_file_suggestions(query, show_on_empty).await;

    // Maps to CC `generateUnifiedSuggestions` (`unifiedSuggestions.ts:151-155`).
    // The empty-query branch concatenates sources in producer order and takes
    // the first 15 rows. It deliberately bypasses Fuse/nucleo scoring.
    if query.is_empty() {
        let mut items = file_suggestions;
        let mut order = items.len();
        for resource in mcp_resources
            .values()
            .flat_map(|resources| resources.iter())
        {
            items.push(mcp_item(resource, order));
            order += 1;
            if items.len() >= MAX_UNIFIED_SUGGESTIONS {
                return items.into_iter().take(MAX_UNIFIED_SUGGESTIONS).collect();
            }
        }
        for agent in agents {
            if agent_matches_query(agent, query) {
                items.push(agent_item(agent, order));
                order += 1;
                if items.len() >= MAX_UNIFIED_SUGGESTIONS {
                    break;
                }
            }
        }
        return items.into_iter().take(MAX_UNIFIED_SUGGESTIONS).collect();
    }

    let mut scored = Vec::new();
    let mut order = 0usize;
    for item in file_suggestions {
        // CC :167-169 — the file producer already ranked these, so the score
        // rides through as-is. `?? 0.5` is the source's "middle score if
        // missing" default, not a sentinel.
        let score = item
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("score"))
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.5);
        scored.push(file_item(item, score, order));
        order += 1;
    }

    // One searcher per query, scoring every non-file row — the source builds
    // one `new Fuse(nonFileSources, ...)` the same way (:174).
    let mut fuse = BitapSearch::new(query, fuse_options());

    let resources = mcp_resources.values().flat_map(|items| items.iter());
    for resource in resources {
        let item = mcp_item(resource, order);
        // CC keys (:178-182) for an MCP row: displayText 2, name 3, server 1,
        // description 1. `uri` is deliberately absent from `keys` — it reaches
        // the match through `displayText`, which is `server:uri` (:141).
        // `name` carries the source's own `resource.name || resource.uri`
        // fallback (:147).
        let name = if resource.name.is_empty() {
            resource.uri.as_str()
        } else {
            resource.name.as_str()
        };
        let score = fuse.compute_score(&[
            FuseKey::new(&item.display_text, 2.0),
            FuseKey::new(name, 3.0),
            FuseKey::new(&resource.server, 1.0),
            FuseKey::new(&item.description, 1.0),
        ]);
        if let Some(score) = score {
            scored.push(ScoredSuggestion {
                item,
                score,
                source_order: order,
            });
        }
        order += 1;
    }
    for agent in agents {
        if !agent_matches_query(agent, query) {
            order += 1;
            continue;
        }
        let item = agent_item(agent, order);
        // CC keys for an agent row: displayText 2, description 1, agentType 3.
        // `name` and `server` do not exist on this source shape, so Fuse never
        // scores them.
        let score = fuse.compute_score(&[
            FuseKey::new(&item.display_text, 2.0),
            FuseKey::new(&item.description, 1.0),
            FuseKey::new(&agent.agent_type, 3.0),
        ]);
        if let Some(score) = score {
            scored.push(ScoredSuggestion {
                item,
                score,
                source_order: order,
            });
        }
        order += 1;
    }

    // CC :196 — one ascending comparator over every source. `source_order`
    // keeps the source's stable `[files, mcp, agents]` ordering for ties.
    scored.sort_by(|left, right| {
        left.score
            .partial_cmp(&right.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.source_order.cmp(&right.source_order))
            .then_with(|| left.item.id.cmp(&right.item.id))
    });
    scored
        .into_iter()
        .take(MAX_UNIFIED_SUGGESTIONS)
        .map(|entry| entry.item)
        .collect()
}

/// Stable source key used by the typeahead request cache.  A source update
/// must re-run the same query even when the user's input did not change.
pub fn source_key(
    mcp_resources: &BTreeMap<String, Vec<ServerResource>>,
    agents: &[AgentDefinition],
) -> String {
    let mut key = String::new();
    for (server, resources) in mcp_resources {
        key.push_str(server);
        key.push(':');
        for resource in resources {
            key.push_str(&resource.uri);
            key.push('\u{0}');
            key.push_str(&resource.name);
            key.push('\u{0}');
            key.push_str(resource.description.as_deref().unwrap_or_default());
            key.push('\u{0}');
            key.push_str(resource.mime_type.as_deref().unwrap_or_default());
            key.push('|');
        }
        key.push(';');
    }
    key.push('#');
    for agent in agents {
        key.push_str(&agent.agent_type);
        key.push('|');
        key.push_str(&agent.when_to_use);
        key.push(';');
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_key_changes_when_mcp_or_agent_inputs_change() {
        let empty = BTreeMap::new();
        assert_ne!(source_key(&empty, &[]), source_key(&empty, &[test_agent()]));
        let mut resources = BTreeMap::new();
        resources.insert(
            "docs".to_string(),
            vec![ServerResource {
                server: "docs".into(),
                uri: "readme".into(),
                name: "README".into(),
                description: None,
                mime_type: None,
            }],
        );
        assert_ne!(source_key(&empty, &[]), source_key(&resources, &[]));
        let mut changed = resources.clone();
        changed.get_mut("docs").unwrap()[0].description = Some("Docs".into());
        assert_ne!(source_key(&resources, &[]), source_key(&changed, &[]));
    }

    fn test_agent() -> AgentDefinition {
        AgentDefinition::new(
            "reviewer",
            "review code",
            crate::tools::agent_tool::load_agents_dir::AgentDefinitionSource::BuiltIn,
        )
    }

    #[test]
    fn agent_rows_keep_source_identity_and_description() {
        let item = agent_item(&test_agent(), 0);
        assert_eq!(item.id, "agent-reviewer");
        assert_eq!(item.command_text, "reviewer");
        assert_eq!(item.description, "review code");
        assert_eq!(
            item.metadata
                .as_ref()
                .and_then(|m| m.get("type"))
                .and_then(serde_json::Value::as_str),
            Some("agent")
        );
    }

    #[test]
    fn agent_filter_uses_source_substring_rule_before_fuzzy_scoring() {
        let agent = test_agent();
        assert!(agent_matches_query(&agent, "view"));
        assert!(agent_matches_query(&agent, "(AGENT)"));
        assert!(!agent_matches_query(&agent, "rwer"));
        assert!(agent_matches_query(&agent, ""));
    }

    /// The scale is the contract. `:196` sorts files, MCP resources and agents
    /// through one ascending comparator, so every source has to land on the
    /// same 0..1 where lower wins — the shared Fuse stand-in for the non-file
    /// rows, the `FileIndex.search` rank for the file rows. When these two
    /// drifted apart, any literal MCP/agent hit scored 0-2 against file scores
    /// in the billions and displaced every file row unconditionally.
    ///
    /// The engine's own behaviour is covered in `utils/fuse.rs`. What this
    /// pins is call-site specific: the `keys` **table** handed to `KeyStore`.
    /// The source configures five keys totalling 10 and Fuse normalizes that
    /// once, so a weight-3 hit scores identically on an MCP row (which carries
    /// four of the five) and an agent row (three). Passing only the keys a row
    /// populates would divide by 7 and by 6 instead — the same key ranked on
    /// two different scales depending on which source it came from.
    #[test]
    fn key_weights_come_from_the_configured_table_not_the_row_shape() {
        let mut fuse = BitapSearch::new("alpha", fuse_options());
        // MCP row: displayText 2, name 3, server 1, description 1.
        let on_mcp_name = fuse
            .compute_score(&[
                FuseKey::new("docs:readme", 2.0),
                FuseKey::new("alpha", 3.0),
                FuseKey::new("docs", 1.0),
                FuseKey::new("", 1.0),
            ])
            .expect("name matches");
        // Agent row: displayText 2, description 1, agentType 3.
        let on_agent_type = fuse
            .compute_score(&[
                FuseKey::new("other (agent)", 2.0),
                FuseKey::new("", 1.0),
                FuseKey::new("alpha", 3.0),
            ])
            .expect("agentType matches");

        assert_eq!(
            on_mcp_name, on_agent_type,
            "a weight-3 hit scores the same whichever source shape carries it"
        );
        assert!(
            (0.0..=1.0).contains(&on_mcp_name),
            "and stays on the scale the file ranks share: {on_mcp_name}"
        );

        // The weight ordering still holds within one row.
        let on_mcp_description = fuse
            .compute_score(&[
                FuseKey::new("docs:readme", 2.0),
                FuseKey::new("other", 3.0),
                FuseKey::new("docs", 1.0),
                FuseKey::new("alpha", 1.0),
            ])
            .expect("description matches");
        assert!(
            on_mcp_name < on_mcp_description,
            "the weight-3 key beats the weight-1 key: {on_mcp_name} vs {on_mcp_description}"
        );
    }

    #[test]
    fn mcp_description_falls_back_from_empty_name_to_uri() {
        let item = mcp_item(
            &ServerResource {
                server: "docs".into(),
                uri: "readme".into(),
                name: String::new(),
                description: None,
                mime_type: None,
            },
            0,
        );
        assert_eq!(item.description, "readme");
    }
}
