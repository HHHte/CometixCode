//! Maps to CC `tools/FileWriteTool/prompt.ts`.

use crate::tools::file_read_tool::prompt::FILE_READ_TOOL_NAME;

pub const FILE_WRITE_TOOL_NAME: &str = "Write";
pub const DESCRIPTION: &str = "Write a file to the local filesystem.";

fn get_pre_read_instruction() -> String {
    format!(
        "\n- If this is an existing file, you MUST use the {FILE_READ_TOOL_NAME} tool first to read the file's contents. This tool will fail if you did not read the file first."
    )
}

pub fn get_write_tool_description() -> String {
    format!(
        "Writes a file to the local filesystem.\n\nUsage:\n- This tool will overwrite the existing file if there is one at the provided path.{}\n- Prefer the Edit tool for modifying existing files — it only sends the diff. Only use this tool to create new files or for complete rewrites.\n- NEVER create documentation files (*.md) or README files unless explicitly requested by the User.\n- Only use emojis if the user explicitly requests it. Avoid writing emojis to files unless asked.",
        get_pre_read_instruction()
    )
}
