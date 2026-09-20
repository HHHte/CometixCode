//! Maps to: CC `utils/permissions/classifierShared.ts`.
//! The source operates on SDK response blocks; local conversation messages do
//! not need a second classifier parser.
use anthropic_sdk::resources::messages::ContentBlock;
use serde::de::DeserializeOwned;

/// Maps to: CC `classifierShared.ts:15-24` `extractToolUseBlock`.
pub fn extract_tool_use_block<'a>(
    content: &'a [ContentBlock],
    tool_name: &str,
) -> Option<&'a ContentBlock> {
    content
        .iter()
        .find(|block| matches!(block, ContentBlock::ToolUse { name, .. } if name == tool_name))
}

/// Maps to: CC `classifierShared.ts:30-39` `parseClassifierResponse`.
/// L1: the concrete Deserialize type carries this caller's required fields;
/// source object schemas also reject positional arrays accepted by Serde structs.
pub fn parse_classifier_response<T: DeserializeOwned>(tool_use_block: &ContentBlock) -> Option<T> {
    let ContentBlock::ToolUse { input, .. } = tool_use_block else {
        return None;
    };
    if !input.is_object() {
        return None;
    }
    serde_json::from_value(input.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct ClassifierResponse {
        should_block: bool,
        reason: String,
    }

    #[test]
    fn extracts_matching_tool_use_block_by_name() {
        let content = serde_json::from_value::<Vec<ContentBlock>>(serde_json::json!([
            {"type":"text","text":"ignored"},
            {"type":"tool_use","id":"toolu_1","name":"classify_result","input":{"should_block":true,"reason":"unsafe"}}
        ])).unwrap();
        let block = extract_tool_use_block(&content, "classify_result").unwrap();
        assert!(matches!(block, ContentBlock::ToolUse { id, .. } if id == "toolu_1"));
        assert!(extract_tool_use_block(&content, "other").is_none());
    }

    #[test]
    fn parses_classifier_response_or_returns_none() {
        for (input, expected) in [
            (
                serde_json::json!({"should_block":false,"reason":"ok"}),
                Some(ClassifierResponse {
                    should_block: false,
                    reason: "ok".into(),
                }),
            ),
            (serde_json::json!({"should_block":false}), None),
            // CC classifierShared.ts:34 + yoloClassifier.ts:252-257: z.object
            // does not deserialize positional fields from arrays.
            (serde_json::json!([false, "ok"]), None),
            (serde_json::json!(null), None),
            (
                serde_json::json!({"should_block":false,"reason":null}),
                None,
            ),
        ] {
            let block = serde_json::from_value::<ContentBlock>(serde_json::json!({"type":"tool_use","id":"toolu_1","name":"classify_result","input":input})).unwrap();
            assert_eq!(
                parse_classifier_response::<ClassifierResponse>(&block),
                expected
            );
        }
    }
}
