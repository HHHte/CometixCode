//! Maps to: CC `components/teams/TeamStatus.tsx:1-53`.

use crate::utils::theme::Theme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct TeamStatusProps {
    /// Team member names; `team-lead` is excluded from the teammate count.
    pub member_names: Vec<String>,
    pub teams_selected: bool,
    pub show_hint: bool,
}

#[component]
pub fn TeamStatus(props: &TeamStatusProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let count = props
        .member_names
        .iter()
        .filter(|name| name.as_str() != "team-lead")
        .count();
    if count == 0 {
        return element! { Fragment }.into_any();
    }
    let theme = hooks.use_context::<Theme>();
    element! { View(flex_direction: FlexDirection::Row) {
        Text(content: format!("{count} {}", if count == 1 { "teammate" } else { "teammates" }), color: theme.background, invert: props.teams_selected)
        #((props.show_hint && props.teams_selected).then(|| element! { Text(content: " · Enter to view".to_string(), dim: true) }))
    }}.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excludes_lead_pluralizes_and_gates_hint() {
        let theme = *crate::utils::theme::current();
        let empty = element! { ContextProvider(value: Context::owned(theme)) { TeamStatus(member_names: vec!["team-lead".to_string()]) } }.render(Some(50)).to_string();
        assert!(empty.trim().is_empty());
        let one = element! { ContextProvider(value: Context::owned(theme)) { TeamStatus(member_names: vec!["team-lead".to_string(), "worker".to_string()], teams_selected: true, show_hint: true) } }.render(Some(50)).to_string();
        assert!(one.contains("1 teammate · Enter to view"));
    }
}
