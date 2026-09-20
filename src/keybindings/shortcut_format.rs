//! Maps to: CC `keybindings/shortcutFormat.ts` and the non-React lookup half
//! of `keybindings/useShortcutDisplay.ts`.
//!
//! The default + user merged loader feeds the non-React fallback, while callers
//! may still inject an explicit parsed binding snapshot.

use super::resolver::get_binding_display_text;
use super::types::{ContextName, ParsedBinding};

/// Maps to: CC `shortcutFormat.ts#getShortcutDisplay`.
pub fn get_shortcut_display_from_bindings(
    action: &str,
    context: &ContextName,
    fallback: &str,
    bindings: &[ParsedBinding],
) -> String {
    get_binding_display_text(action, context, bindings).unwrap_or_else(|| fallback.to_string())
}

/// Maps to: CC `useShortcutDisplay(action, context, fallback)` when no React
/// keybinding context/user overrides are available.
pub fn get_shortcut_display(action: &str, context: &ContextName, fallback: &str) -> String {
    get_shortcut_display_from_bindings(
        action,
        context,
        fallback,
        &crate::keybindings::load_user_bindings::load_keybindings_sync(),
    )
}

/// Convenience adapter for UI props that carry official context strings.
pub fn get_shortcut_display_for_context_name(
    action: &str,
    context: &str,
    fallback: &str,
) -> String {
    let context = ContextName::from_official_str(context);
    get_shortcut_display(action, &context, fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keybindings::parser::parse_chord;

    fn binding(keys: &str, action: Option<&str>, context: ContextName) -> ParsedBinding {
        ParsedBinding {
            chord: parse_chord(keys),
            action: action.map(str::to_string),
            context,
        }
    }

    #[test]
    fn shortcut_display_returns_configured_binding_or_fallback() {
        let bindings = vec![binding(
            "ctrl+o",
            Some("app:toggleTranscript"),
            ContextName::Global,
        )];

        assert_eq!(
            get_shortcut_display_from_bindings(
                "app:toggleTranscript",
                &ContextName::Global,
                "fallback",
                &bindings,
            ),
            "ctrl+o"
        );
        assert_eq!(
            get_shortcut_display_from_bindings(
                "missing",
                &ContextName::Global,
                "fallback",
                &bindings
            ),
            "fallback"
        );
    }

    #[test]
    fn shortcut_display_uses_default_bindings_for_official_context_names() {
        assert_eq!(
            get_shortcut_display_for_context_name("confirm:no", "Confirmation", "Esc"),
            "Esc"
        );
        assert_eq!(
            get_shortcut_display_for_context_name("app:toggleTranscript", "Global", "ctrl+o"),
            "ctrl+o"
        );
    }
}
