//! Maps to: CC commands/plugin/UnifiedInstalledCell.tsx.
use super::unified_types::{UnifiedInstalledItem, UnifiedInstalledKind};
use crate::services::mcp::types::McpServerConnectionType;
use iocraft::prelude::*;
#[derive(Default, Props)]
pub struct UnifiedInstalledCellProps {
    pub item: Option<UnifiedInstalledItem>,
    pub is_selected: bool,
}
/// Maps to: CC UnifiedInstalledCell.tsx:14-151#UnifiedInstalledCell.
#[component]
pub fn UnifiedInstalledCell(
    props: &UnifiedInstalledCellProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let Some(item) = &props.item else {
        return element! {View}.into_any();
    };
    let selected = props.is_selected;
    let figures = crate::constants::figures::figures();
    let (icon, status, status_color, tag, indented) = match &item.kind {
        UnifiedInstalledKind::Plugin {
            is_enabled,
            errors,
            pending_toggle,
            ..
        } => {
            let (icon, text, color) = if let Some(pending) = pending_toggle {
                (
                    figures.arrow_right,
                    if pending == "will-enable" {
                        "will enable".into()
                    } else {
                        "will disable".into()
                    },
                    theme.suggestion,
                )
            } else if !errors.is_empty() {
                (
                    figures.cross,
                    format!(
                        "{} {}",
                        errors.len(),
                        if errors.len() == 1 { "error" } else { "errors" }
                    ),
                    theme.error,
                )
            } else if !is_enabled {
                (figures.radio_off, "disabled".into(), theme.inactive)
            } else {
                (figures.tick, "enabled".into(), theme.success)
            };
            (icon, text, color, "Plugin", false)
        }
        UnifiedInstalledKind::FailedPlugin { errors } => (
            figures.cross,
            format!(
                "failed to load · {} {}",
                errors.len(),
                if errors.len() == 1 { "error" } else { "errors" }
            ),
            theme.error,
            "Plugin",
            false,
        ),
        UnifiedInstalledKind::FlaggedPlugin { .. } => (
            figures.warning,
            "removed".into(),
            theme.warning,
            "Plugin",
            false,
        ),
        UnifiedInstalledKind::Mcp { client, indented } => {
            let (icon, text, color) = match client.official_client_type() {
                McpServerConnectionType::Connected => (figures.tick, "connected", theme.success),
                McpServerConnectionType::Disabled => {
                    (figures.radio_off, "disabled", theme.inactive)
                }
                McpServerConnectionType::Pending => {
                    (figures.radio_off, "connecting…", theme.inactive)
                }
                McpServerConnectionType::NeedsAuth => {
                    (figures.triangle_up_outline, "Enter to auth", theme.warning)
                }
                _ => (figures.cross, "failed", theme.error),
            };
            (icon, text.into(), color, "MCP", *indented)
        }
    };
    element!{View(flex_direction:FlexDirection::Row){Text(content:if selected{format!("{} ",figures.pointer)}else{"  ".into()},color:selected.then_some(theme.suggestion)) #(indented.then(||element!{Text(content:"└ ",dim:!selected)})) Text(content:item.name.clone(),color:selected.then_some(theme.suggestion)) Text(content:" ",dim:!selected) Text(content:tag,background_color:theme.user_message_bg,dim:!selected) #(item.marketplace.as_ref().map(|m|element!{Text(content:format!(" · {m}"),dim:true)})) Text(content:" · ",dim:!selected) Text(content:icon,color:status_color,dim:!selected) Text(content:format!(" {status}"),dim:!selected)}}.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::plugin::{LoadedPlugin, PluginError};
    #[test]
    fn pending_toggle_precedes_errors_and_keeps_canonical_cell_copy() {
        let item = UnifiedInstalledItem {
            id: "alpha@market".into(),
            name: "alpha".into(),
            description: None,
            marketplace: Some("market".into()),
            scope: "user".into(),
            kind: UnifiedInstalledKind::Plugin {
                plugin: LoadedPlugin::default(),
                is_enabled: false,
                errors: vec![PluginError::GenericError {
                    source: "alpha@market".into(),
                    plugin: None,
                    error: "missing".into(),
                }],
                pending_enable: None,
                pending_update: false,
                pending_toggle: Some("will-enable".into()),
            },
        };
        let mut row = element! {ContextProvider(value:Context::owned(*crate::utils::theme::current())){UnifiedInstalledCell(item:Some(item),is_selected:true)}};
        let text = row.to_string();
        assert!(
            text.contains(&format!(
                "{} alpha Plugin · market · {} will enable",
                crate::constants::figures::figures().pointer,
                crate::constants::figures::figures().arrow_right
            )),
            "{text}"
        );
        assert!(!text.contains("1 error"));
        assert!(!text.contains("disabled"));
    }
}
