//! UI-only port of official `ScheduleCronTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

use super::{CreateOutput, DeleteOutput, ListJob, ListOutput};

// ─── safeParse stand-ins (`UserToolSuccessMessage.tsx:80`) ───────────────

fn optional_bool(
    map: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<Option<bool>> {
    match map.get(key) {
        None => Some(None),
        Some(serde_json::Value::Bool(value)) => Some(Some(*value)),
        Some(_) => None,
    }
}

/// Strict `CronCreateTool.ts:45-52` output contract.
pub(crate) fn parse_create_output(value: &serde_json::Value) -> Option<CreateOutput> {
    let map = value.as_object()?;
    Some(CreateOutput {
        id: map.get("id")?.as_str()?.to_string(),
        human_schedule: map.get("humanSchedule")?.as_str()?.to_string(),
        recurring: map.get("recurring")?.as_bool()?,
        durable: optional_bool(map, "durable")?,
    })
}

pub(crate) fn create_output_to_value(output: &CreateOutput) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "id".to_string(),
        serde_json::Value::String(output.id.clone()),
    );
    map.insert(
        "humanSchedule".to_string(),
        serde_json::Value::String(output.human_schedule.clone()),
    );
    map.insert(
        "recurring".to_string(),
        serde_json::Value::Bool(output.recurring),
    );
    if let Some(durable) = output.durable {
        map.insert("durable".to_string(), serde_json::Value::Bool(durable));
    }
    serde_json::Value::Object(map)
}

/// Strict `CronDeleteTool.ts:27-31` output contract.
pub(crate) fn parse_delete_output(value: &serde_json::Value) -> Option<DeleteOutput> {
    Some(DeleteOutput {
        id: value.as_object()?.get("id")?.as_str()?.to_string(),
    })
}

pub(crate) fn delete_output_to_value(output: &DeleteOutput) -> serde_json::Value {
    serde_json::json!({ "id": output.id })
}

/// Strict `CronListTool.ts:20-33` output contract — a malformed job rejects
/// the whole payload, as Zod does.
pub(crate) fn parse_list_output(value: &serde_json::Value) -> Option<ListOutput> {
    let jobs = value
        .as_object()?
        .get("jobs")?
        .as_array()?
        .iter()
        .map(|job| {
            let map = job.as_object()?;
            Some(ListJob {
                id: map.get("id")?.as_str()?.to_string(),
                cron: map.get("cron")?.as_str()?.to_string(),
                human_schedule: map.get("humanSchedule")?.as_str()?.to_string(),
                prompt: map.get("prompt")?.as_str()?.to_string(),
                recurring: optional_bool(map, "recurring")?,
                durable: optional_bool(map, "durable")?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ListOutput { jobs })
}

pub(crate) fn list_output_to_value(output: &ListOutput) -> serde_json::Value {
    serde_json::json!({
        "jobs": output
            .jobs
            .iter()
            .map(|job| {
                let mut map = serde_json::Map::new();
                map.insert("id".to_string(), serde_json::Value::String(job.id.clone()));
                map.insert(
                    "cron".to_string(),
                    serde_json::Value::String(job.cron.clone()),
                );
                map.insert(
                    "humanSchedule".to_string(),
                    serde_json::Value::String(job.human_schedule.clone()),
                );
                map.insert(
                    "prompt".to_string(),
                    serde_json::Value::String(job.prompt.clone()),
                );
                if let Some(recurring) = job.recurring {
                    map.insert("recurring".to_string(), serde_json::Value::Bool(recurring));
                }
                if let Some(durable) = job.durable {
                    map.insert("durable".to_string(), serde_json::Value::Bool(durable));
                }
                serde_json::Value::Object(map)
            })
            .collect::<Vec<_>>(),
    })
}

// ─── renderToolResultMessage raw entries (dispatch) ──────────────────────

/// Maps to: CC `ScheduleCronTool/UI.tsx` `renderCreateResultMessage` —
/// "Scheduled {id} ({humanSchedule})"; missing/rejected raw renders nothing
/// (`UserToolSuccessMessage.tsx:72,81`).
pub(crate) fn render_create_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_create_output) else {
        return Vec::new();
    };
    vec![ToolRenderLine::new(
        format!("Scheduled {} ({})", output.id, output.human_schedule),
        ToolRenderTone::Normal,
    )]
}

/// Maps to: CC `ScheduleCronTool/UI.tsx` `renderDeleteResultMessage` —
/// "Cancelled {id}".
pub(crate) fn render_delete_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_delete_output) else {
        return Vec::new();
    };
    vec![ToolRenderLine::new(
        format!("Cancelled {}", output.id),
        ToolRenderTone::Normal,
    )]
}

/// Maps to: CC `ScheduleCronTool/UI.tsx` `renderListResultMessage` — "No
/// scheduled jobs" when empty, else one "{id} {humanSchedule}" line per job.
pub(crate) fn render_list_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_list_output) else {
        return Vec::new();
    };
    if output.jobs.is_empty() {
        return vec![ToolRenderLine::new(
            "No scheduled jobs",
            ToolRenderTone::Inactive,
        )];
    }
    output
        .jobs
        .iter()
        .map(|job| {
            ToolRenderLine::new(
                format!("{} {}", job.id, job.human_schedule),
                ToolRenderTone::Normal,
            )
        })
        .collect()
}

pub(crate) fn cron_create_tool_use_summary(input: &serde_json::Value) -> Option<String> {
    let cron = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["cron"],
    )
    .unwrap_or_default();
    match crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["prompt"],
    ) {
        // CC UI.tsx:14: `truncate(input.prompt, 60, true)` — singleLine.
        Some(prompt) if !prompt.is_empty() => Some(format!(
            "{cron}: {}",
            crate::utils::truncate::truncate(&prompt, 60, true)
        )),
        _ => Some(cron),
    }
}
