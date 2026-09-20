//! Maps to: CC `components/tasks/BackgroundTaskStatus.tsx:1-310`.

use super::task_status_utils::{FooterTaskProjection, should_hide_tasks_footer};
use crate::utils::theme::{Theme, ThemeColorKey};
use iocraft::prelude::*;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeammatePillData {
    pub name: String,
    pub color: Option<ThemeColorKey>,
    pub is_idle: bool,
    pub task_id: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackgroundTaskStatusData {
    pub teammate_pills: Vec<TeammatePillData>,
    pub footer_tasks: Vec<FooterTaskProjection>,
    pub summary_label: String,
    pub summary_needs_cta: bool,
    pub viewing_agent_task_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HorizontalWindow {
    pub start: usize,
    pub end: usize,
    pub left: bool,
    pub right: bool,
}

pub fn calculate_horizontal_window(
    widths: &[usize],
    available: usize,
    arrow_width: usize,
    selected: usize,
) -> HorizontalWindow {
    if widths.is_empty() {
        return HorizontalWindow {
            start: 0,
            end: 0,
            left: false,
            right: false,
        };
    }
    let selected = selected.min(widths.len() - 1);
    let mut start = selected;
    let mut end = selected + 1;
    let mut used = widths[selected];
    loop {
        let left_cost = if start > 0 {
            widths[start - 1]
        } else {
            usize::MAX
        };
        let right_cost = if end < widths.len() {
            widths[end]
        } else {
            usize::MAX
        };
        let reserve =
            usize::from(start > 0) * arrow_width + usize::from(end < widths.len()) * arrow_width;
        if left_cost <= right_cost
            && left_cost != usize::MAX
            && used + left_cost + reserve <= available
        {
            start -= 1;
            used += left_cost;
        } else if right_cost != usize::MAX && used + right_cost + reserve <= available {
            used += right_cost;
            end += 1;
        } else {
            break;
        }
    }
    HorizontalWindow {
        start,
        end,
        left: start > 0,
        right: end < widths.len(),
    }
}

#[derive(Default, Props)]
pub struct BackgroundTaskStatusProps {
    pub data: Option<BackgroundTaskStatusData>,
    pub tasks_selected: bool,
    pub is_viewing_teammate: bool,
    pub teammate_footer_index: usize,
    pub is_leader_idle: bool,
    pub show_spinner_tree: bool,
    pub columns: Option<usize>,
}

#[component]
pub fn BackgroundTaskStatus(
    props: &BackgroundTaskStatusProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let Some(data) = props.data.as_ref() else {
        return element! { Fragment }.into_any();
    };
    let all_teammates = !props.show_spinner_tree
        && !data.footer_tasks.is_empty()
        && data
            .footer_tasks
            .iter()
            .all(|task| task.is_in_process_teammate);
    if all_teammates || (!props.show_spinner_tree && props.is_viewing_teammate) {
        let mut teammates = data.teammate_pills.clone();
        teammates.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });
        if !props.tasks_selected {
            teammates.sort_by_key(|teammate| teammate.is_idle);
        }
        let mut pills = vec![(
            "main".to_string(),
            None,
            props.is_leader_idle,
            None::<String>,
        )];
        pills.extend(teammates.into_iter().map(|teammate| {
            (
                teammate.name,
                teammate.color,
                teammate.is_idle,
                Some(teammate.task_id),
            )
        }));
        let widths = pills
            .iter()
            .enumerate()
            .map(|(index, (name, _, _, _))| {
                UnicodeWidthStr::width(format!("@{name}").as_str()) + usize::from(index > 0)
            })
            .collect::<Vec<_>>();
        let selected = if props.tasks_selected {
            props
                .teammate_footer_index
                .min(pills.len().saturating_sub(1))
        } else {
            0
        };
        let available = props.columns.unwrap_or(80).saturating_sub(24).max(20);
        let window = calculate_horizontal_window(&widths, available, 2, selected);
        let viewed = data.viewing_agent_task_id.as_deref();
        let theme = hooks.use_context::<Theme>();
        let visible = pills[window.start..window.end]
            .iter()
            .enumerate()
            .map(|(visible_index, (name, color, idle, task_id))| {
                let index = window.start + visible_index;
                let is_selected = props.tasks_selected && selected == index;
                let is_viewed = if index == 0 {
                    viewed.is_none()
                } else {
                    task_id.as_deref() == viewed
                };
                let mut content = MixedTextContent::new(format!(
                    "{}@{name}",
                    if visible_index > 0 { " " } else { "" }
                ));
                content.color = if is_selected {
                    Some(theme.inverse_text)
                } else {
                    color.map(|key| theme.color(key))
                };
                content.background_color = if is_selected {
                    color.map(|key| theme.color(key)).or(Some(theme.background))
                } else {
                    None
                };
                content.weight = if is_viewed {
                    Weight::Bold
                } else if *idle {
                    Weight::Light
                } else {
                    Weight::Normal
                };
                content.invert = is_selected && color.is_none();
                element! { MixedText(contents: vec![content]) }
            })
            .collect::<Vec<_>>();
        return element! { View(flex_direction: FlexDirection::Row) {
            #(window.left.then(|| element! { Text(content: format!("{} ", crate::constants::figures::get().arrow_left), dim: true) }))
            #(visible)
            #(window.right.then(|| element! { Text(content: format!(" {}", crate::constants::figures::get().arrow_right), dim: true) }))
            Text(content: " · shift + ↓ to expand".to_string(), dim: true)
        }}.into_any();
    }
    if should_hide_tasks_footer(&data.footer_tasks, props.show_spinner_tree)
        || data.footer_tasks.is_empty()
    {
        return element! { Fragment }.into_any();
    }
    let theme = hooks.use_context::<Theme>();
    element! { View(flex_direction: FlexDirection::Row) {
        Text(content: data.summary_label.clone(), color: theme.background, invert: props.tasks_selected)
        #(data.summary_needs_cta.then(|| element! { Text(content: format!(" · {} to view", crate::constants::figures::get().arrow_down), dim: true) }))
    }}.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_keeps_selection_visible_and_reports_arrows() {
        let window = calculate_horizontal_window(&[8, 8, 8, 8], 18, 2, 2);
        assert!(window.start <= 2 && window.end > 2);
        assert!(window.left || window.right);
    }
    #[test]
    fn summary_and_team_modes_render() {
        let theme = *crate::utils::theme::current();
        let summary = BackgroundTaskStatusData {
            footer_tasks: vec![FooterTaskProjection {
                is_background: true,
                ..Default::default()
            }],
            summary_label: "2 tasks".to_string(),
            summary_needs_cta: true,
            ..Default::default()
        };
        let text = element! { ContextProvider(value: Context::owned(theme)) { BackgroundTaskStatus(data: Some(summary)) } }.render(Some(80)).to_string();
        assert!(text.contains("2 tasks · ↓ to view"));
        let team = BackgroundTaskStatusData {
            teammate_pills: vec![TeammatePillData {
                name: "worker".to_string(),
                color: Some(ThemeColorKey::AgentBlue),
                is_idle: false,
                task_id: "1".to_string(),
            }],
            footer_tasks: vec![FooterTaskProjection {
                is_background: true,
                is_in_process_teammate: true,
                ..Default::default()
            }],
            ..Default::default()
        };
        let text = element! { ContextProvider(value: Context::owned(theme)) { BackgroundTaskStatus(data: Some(team)) } }.render(Some(100)).to_string();
        assert!(text.contains("@main @worker · shift + ↓ to expand"));
    }
}
