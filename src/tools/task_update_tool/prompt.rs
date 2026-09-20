//! Maps to CC `tools/TaskUpdateTool/constants.ts` and `prompt.ts`.

pub const TASK_UPDATE_TOOL_NAME: &str = "TaskUpdate";
pub const DESCRIPTION: &str = "Update a task in the task list";

pub const PROMPT: &str = r#"Use this tool to update a task in the task list.

## When to Use This Tool

**Mark tasks as resolved:**
- When you have completed the work described in a task
- When a task is no longer needed or has been superseded
- IMPORTANT: Always mark your assigned tasks as resolved when you finish them
- After resolving, call TaskList to find your next task

- ONLY mark a task as completed when you have FULLY accomplished it
- If you encounter errors, blockers, or cannot finish, keep the task as in_progress

**Delete tasks:**
- When a task is no longer relevant or was created in error
- Setting status to `deleted` permanently removes the task

**Update task details:**
- When requirements change or become clearer
- When establishing dependencies between tasks

## Fields You Can Update

- **status**: The task status
- **subject**: Change the task title
- **description**: Change the task description
- **activeForm**: Present continuous form shown in spinner when in_progress
- **owner**: Change the task owner
- **metadata**: Merge metadata keys into the task
- **addBlocks**: Mark tasks that cannot start until this one completes
- **addBlockedBy**: Mark tasks that must complete before this one can start

## Status Workflow

Status progresses: `pending` → `in_progress` → `completed`

Use `deleted` to permanently remove a task.

## Staleness

Make sure to read a task's latest state using `TaskGet` before updating it.
"#;
