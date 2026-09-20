//! LSP tool input schema helpers.
//!
//! Maps to:
//! - CC `tools/LSPTool/schemas.ts`
//! - CC `tools/LSPTool/LSPTool.ts` `getMethodAndParams(...)`

use std::path::{Path, PathBuf};

/// Maps to: CC `LSPToolInput['operation']` discriminated-union literals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LspOperation {
    GoToDefinition,
    FindReferences,
    Hover,
    DocumentSymbol,
    WorkspaceSymbol,
    GoToImplementation,
    PrepareCallHierarchy,
    IncomingCalls,
    OutgoingCalls,
}

impl LspOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GoToDefinition => "goToDefinition",
            Self::FindReferences => "findReferences",
            Self::Hover => "hover",
            Self::DocumentSymbol => "documentSymbol",
            Self::WorkspaceSymbol => "workspaceSymbol",
            Self::GoToImplementation => "goToImplementation",
            Self::PrepareCallHierarchy => "prepareCallHierarchy",
            Self::IncomingCalls => "incomingCalls",
            Self::OutgoingCalls => "outgoingCalls",
        }
    }

    fn initial_method(self) -> &'static str {
        match self {
            Self::GoToDefinition => "textDocument/definition",
            Self::FindReferences => "textDocument/references",
            Self::Hover => "textDocument/hover",
            Self::DocumentSymbol => "textDocument/documentSymbol",
            Self::WorkspaceSymbol => "workspace/symbol",
            Self::GoToImplementation => "textDocument/implementation",
            Self::PrepareCallHierarchy | Self::IncomingCalls | Self::OutgoingCalls => {
                "textDocument/prepareCallHierarchy"
            }
        }
    }
}

impl TryFrom<&str> for LspOperation {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "goToDefinition" => Ok(Self::GoToDefinition),
            "findReferences" => Ok(Self::FindReferences),
            "hover" => Ok(Self::Hover),
            "documentSymbol" => Ok(Self::DocumentSymbol),
            "workspaceSymbol" => Ok(Self::WorkspaceSymbol),
            "goToImplementation" => Ok(Self::GoToImplementation),
            "prepareCallHierarchy" => Ok(Self::PrepareCallHierarchy),
            "incomingCalls" => Ok(Self::IncomingCalls),
            "outgoingCalls" => Ok(Self::OutgoingCalls),
            other => Err(format!("Invalid LSP operation: {other}")),
        }
    }
}

/// Maps to: CC `tools/LSPTool/schemas.ts` `isValidLSPOperation(...)`.
pub fn is_valid_lsp_operation(operation: &str) -> bool {
    LspOperation::try_from(operation).is_ok()
}

/// Maps to: CC `tools/LSPTool/schemas.ts` `LSPToolInput`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspToolInput {
    pub operation: LspOperation,
    pub file_path: String,
    /// 1-based editor line number.
    pub line: u32,
    /// 1-based editor character offset.
    pub character: u32,
}

/// Maps to the discriminated-union validation done by CC `validateInput(...)`.
pub fn parse_lsp_tool_input(input: &serde_json::Value) -> Result<LspToolInput, String> {
    let operation = input
        .get("operation")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Missing required field: operation".to_string())
        .and_then(LspOperation::try_from)?;
    let file_path = input
        .get("filePath")
        .or_else(|| input.get("file_path"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Missing required field: filePath".to_string())?
        .to_string();
    let line = positive_u32_field(input, "line")?;
    let character = positive_u32_field(input, "character")?;

    Ok(LspToolInput {
        operation,
        file_path,
        line,
        character,
    })
}

fn positive_u32_field(input: &serde_json::Value, field: &str) -> Result<u32, String> {
    match input.get(field).and_then(|value| value.as_u64()) {
        Some(value) if value > 0 && value <= u32::MAX as u64 => Ok(value as u32),
        Some(_) => Err(format!("Field {field} must be a positive integer")),
        None => Err(format!("Missing required field: {field}")),
    }
}

/// Maps to the return value of CC `getMethodAndParams(...)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LspMethodAndParams {
    pub method: &'static str,
    pub params: serde_json::Value,
}

/// Maps to: CC `tools/LSPTool/LSPTool.ts` `getMethodAndParams(...)`.
pub fn get_method_and_params(input: &LspToolInput, absolute_path: &Path) -> LspMethodAndParams {
    let uri = path_to_file_url(absolute_path);
    let position = serde_json::json!({
        "line": input.line - 1,
        "character": input.character - 1,
    });

    let params = match input.operation {
        LspOperation::WorkspaceSymbol => serde_json::json!({ "query": "" }),
        LspOperation::DocumentSymbol => serde_json::json!({
            "textDocument": { "uri": uri },
        }),
        LspOperation::FindReferences => serde_json::json!({
            "textDocument": { "uri": uri },
            "position": position,
            "context": { "includeDeclaration": true },
        }),
        _ => serde_json::json!({
            "textDocument": { "uri": uri },
            "position": position,
        }),
    };

    LspMethodAndParams {
        method: input.operation.initial_method(),
        params,
    }
}

fn path_to_file_url(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        PathBuf::from(path)
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut normalized = absolute.to_string_lossy().replace('\\', "/");
    if !normalized.starts_with('/') {
        normalized = format!("/{normalized}");
    }
    format!("file://{}", percent_encode_path(&normalized))
}

fn percent_encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_lsp_operation_matches_official_literals() {
        for operation in crate::tools::lsp_tool::prompt::OPERATIONS {
            assert!(is_valid_lsp_operation(operation), "{operation}");
        }
        assert!(!is_valid_lsp_operation("rename"));
    }

    #[test]
    fn parse_lsp_input_requires_positive_editor_position() {
        let input = serde_json::json!({
            "operation": "hover",
            "filePath": "src/main.rs",
            "line": 1,
            "character": 2,
        });
        let parsed = parse_lsp_tool_input(&input).unwrap();
        assert_eq!(parsed.operation, LspOperation::Hover);
        assert_eq!(parsed.file_path, "src/main.rs");
        assert_eq!(parsed.line, 1);
        assert_eq!(parsed.character, 2);

        let invalid = serde_json::json!({
            "operation": "hover",
            "filePath": "src/main.rs",
            "line": 0,
            "character": 1,
        });
        assert!(parse_lsp_tool_input(&invalid).unwrap_err().contains("line"));
    }

    #[test]
    fn method_params_convert_editor_position_to_lsp_zero_based() {
        let input = LspToolInput {
            operation: LspOperation::FindReferences,
            file_path: "/tmp/project/src main.rs".to_string(),
            line: 9,
            character: 3,
        };
        let mapped = get_method_and_params(&input, Path::new(&input.file_path));

        assert_eq!(mapped.method, "textDocument/references");
        assert_eq!(mapped.params["position"]["line"], 8);
        assert_eq!(mapped.params["position"]["character"], 2);
        assert_eq!(mapped.params["context"]["includeDeclaration"], true);
        assert_eq!(
            mapped.params["textDocument"]["uri"],
            "file:///tmp/project/src%20main.rs"
        );
    }

    #[test]
    fn workspace_symbol_uses_empty_query_like_official() {
        let input = LspToolInput {
            operation: LspOperation::WorkspaceSymbol,
            file_path: "/tmp/project/src/main.rs".to_string(),
            line: 1,
            character: 1,
        };
        let mapped = get_method_and_params(&input, Path::new(&input.file_path));
        assert_eq!(mapped.method, "workspace/symbol");
        assert_eq!(mapped.params, serde_json::json!({ "query": "" }));
    }

    #[test]
    fn call_hierarchy_operations_prepare_first_like_official() {
        for operation in [LspOperation::IncomingCalls, LspOperation::OutgoingCalls] {
            let input = LspToolInput {
                operation,
                file_path: "/tmp/project/src/main.rs".to_string(),
                line: 1,
                character: 1,
            };
            let mapped = get_method_and_params(&input, Path::new(&input.file_path));
            assert_eq!(mapped.method, "textDocument/prepareCallHierarchy");
        }
    }
}
