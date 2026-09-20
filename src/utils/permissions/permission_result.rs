//! Maps to: CC `utils/permissions/PermissionResult.ts`.
//!
//! CC's file is exactly this shape: the decision/result types are DEFINED in
//! `types/permissions.ts` ("Types extracted to src/types/permissions.ts to
//! break import cycles") and re-exported here "for backwards compatibility",
//! with `getRuleBehaviorDescription` as the file's one definition. An earlier
//! #142 note claimed this CC file did not exist and marked the shim removable —
//! wrong on both counts: the re-export surface is itself the 1:1 mapping, and
//! it stays.

pub use crate::types::permissions::{
    PendingClassifierCheck, PermissionCommandMetadata, PermissionDecision,
    PermissionDecisionReason, PermissionMetadata, PermissionResult, SandboxOverrideReason,
};

/// Maps to: CC `PermissionResult.ts#getRuleBehaviorDescription` — defined in
/// THIS file upstream, not in `types/permissions.ts`.
pub fn get_rule_behavior_description(behavior: &str) -> &'static str {
    match behavior {
        "allow" => "allowed",
        "deny" => "denied",
        _ => "asked for confirmation for",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::{
        PermissionBehavior, PermissionRule, PermissionRuleSource, PermissionRuleValue,
    };
    use std::collections::BTreeMap;

    #[test]
    fn permission_result_behavior_description_matches_official() {
        assert_eq!(get_rule_behavior_description("allow"), "allowed");
        assert_eq!(get_rule_behavior_description("deny"), "denied");
        assert_eq!(
            get_rule_behavior_description("ask"),
            "asked for confirmation for"
        );
        assert_eq!(
            get_rule_behavior_description("passthrough"),
            "asked for confirmation for"
        );
    }

    #[test]
    fn permission_decision_reason_serializes_official_rule_shape() {
        let reason = PermissionDecisionReason::Rule {
            rule: PermissionRule {
                source: PermissionRuleSource::LocalSettings,
                rule_behavior: PermissionBehavior::Allow,
                rule_value: PermissionRuleValue::new("Bash", Some("ls:*".to_string())),
            },
        };
        let value = serde_json::to_value(&reason).unwrap();
        assert_eq!(value["type"], "rule");
        assert_eq!(value["rule"]["source"], "localSettings");
        assert_eq!(value["rule"]["ruleValue"]["toolName"], "Bash");
    }

    #[test]
    fn permission_result_carries_passthrough_and_subcommand_results() {
        let mut reasons = BTreeMap::new();
        reasons.insert(
            "git status".to_string(),
            Box::new(PermissionResult::Allow {
                updated_input: None,
                user_modified: Some(false),
                decision_reason: None,
                tool_use_id: None,
                accept_feedback: None,
                content_blocks: Vec::new(),
            }),
        );
        let result = PermissionResult::Passthrough {
            message: "compound".to_string(),
            decision_reason: Some(PermissionDecisionReason::SubcommandResults { reasons }),
            suggestions: Vec::new(),
            blocked_path: None,
            pending_classifier_check: None,
        };
        assert_eq!(result.behavior(), "passthrough");
    }

    /// #142 step 3. CC's `PermissionResult` IS `PermissionDecision |
    /// passthrough` (`types/permissions.ts:251-266`) — a flat union, so the
    /// allow/ask/deny values are the decision values verbatim. Rust cannot
    /// share variants, so the composition is carried by the converter pair;
    /// this pins that it is lossless in both directions for every field,
    /// which is what the duplicated arms were NOT before (Allow lacked
    /// toolUseID/acceptFeedback/contentBlocks, Ask lacked updatedInput/
    /// isBashSecurityCheckForMisparsing/contentBlocks, Deny lacked toolUseID).
    #[test]
    fn permission_decision_and_result_round_trip_every_field() {
        let decisions = vec![
            PermissionDecision::Allow {
                updated_input: Some(serde_json::json!({"command": "ls"})),
                user_modified: Some(true),
                decision_reason: Some(PermissionDecisionReason::Other {
                    reason: "probe".to_string(),
                }),
                tool_use_id: Some("toolu_1".to_string()),
                accept_feedback: Some("looks good".to_string()),
                content_blocks: vec![serde_json::json!({"type": "text", "text": "block"})],
            },
            PermissionDecision::Ask {
                message: "Approve?".to_string(),
                updated_input: Some(serde_json::json!({"command": "rm"})),
                decision_reason: None,
                suggestions: Vec::new(),
                blocked_path: Some("/etc".to_string()),
                metadata: None,
                is_bash_security_check_for_misparsing: true,
                pending_classifier_check: None,
                content_blocks: vec![serde_json::json!({"type": "text", "text": "why"})],
            },
            PermissionDecision::Deny {
                message: "No.".to_string(),
                decision_reason: PermissionDecisionReason::Other {
                    reason: "denied".to_string(),
                },
                tool_use_id: Some("toolu_2".to_string()),
            },
        ];

        for decision in decisions {
            let result = PermissionResult::from(decision.clone());
            assert_eq!(
                result.behavior(),
                match &decision {
                    PermissionDecision::Allow { .. } => "allow",
                    PermissionDecision::Ask { .. } => "ask",
                    PermissionDecision::Deny { .. } => "deny",
                }
            );
            // The JSON identity CC's flat union implies: same behavior tag,
            // same field names, same values.
            assert_eq!(
                serde_json::to_value(&result).unwrap(),
                serde_json::to_value(&decision).unwrap()
            );
            assert_eq!(PermissionDecision::try_from(result), Ok(decision));
        }

        // Passthrough is the one value the result adds over the union, so it
        // has no decision to project back to.
        let passthrough = PermissionResult::Passthrough {
            message: "tool has no opinion".to_string(),
            decision_reason: None,
            suggestions: Vec::new(),
            blocked_path: None,
            pending_classifier_check: None,
        };
        assert_eq!(
            PermissionDecision::try_from(passthrough.clone()),
            Err(passthrough)
        );
    }
}
