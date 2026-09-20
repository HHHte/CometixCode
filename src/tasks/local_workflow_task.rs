//! Local workflow task state (ant WORKFLOW_SCRIPTS).
//!
//! Maps to: CC `tasks/LocalWorkflowTask/LocalWorkflowTask.ts:1-4`.
//!
//! NO-SOURCE SEAM: the CC 2.1.88 rebuild carries only a 4-line
//! `@generated-stub` for this file (missing from the sourcemap; the type below
//! is that stub verbatim). The real LocalWorkflowTask runtime is ant-only
//! (`WORKFLOW_SCRIPTS`, statically null in external builds — see
//! `docs/reverse/WORKFLOW_TOOL.md`) and has no portable source; do not invent
//! it here.

/// Maps to: CC generated-stub `LocalWorkflowTaskState` —
/// `{ status: 'pending' | 'running' | 'completed' | 'failed'; workflowName?: string }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalWorkflowTaskState {
    pub status: String,
    pub workflow_name: Option<String>,
}
