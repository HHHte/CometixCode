//! Maps to CC `constants/toolLimits.ts`.

/// System-wide character cap applied before oversized tool results are stored.
pub const DEFAULT_MAX_RESULT_SIZE_CHARS: usize = 50_000;
/// Maximum estimated tokens in one tool result.
pub const MAX_TOOL_RESULT_TOKENS: usize = 100_000;
/// Conservative byte-per-token estimate.
pub const BYTES_PER_TOKEN: usize = 4;
/// Byte cap derived from [`MAX_TOOL_RESULT_TOKENS`].
pub const MAX_TOOL_RESULT_BYTES: usize = MAX_TOOL_RESULT_TOKENS * BYTES_PER_TOKEN;
/// Aggregate character budget for tool-result blocks in one user message.
pub const MAX_TOOL_RESULTS_PER_MESSAGE_CHARS: usize = 200_000;
/// Maximum terminal-column width of compact tool-use summaries.
pub const TOOL_SUMMARY_MAX_LENGTH: usize = 50;
