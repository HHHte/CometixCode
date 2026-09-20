//! Maps to: CC `utils/ide.ts` helpers used by IDE onboarding dialogs.
//!
//! This module is a pure/read-only boundary for the dialog slice. It does not
//! detect processes, inspect lockfiles, install IDE extensions, call MCP IDE RPC,
//! or write global config.

use crate::utils::config::GlobalConfig;
use std::collections::HashMap;
use std::path::Path;

/// Maps to: CC `utils/ide.ts` `IdeType`.
pub type IdeType = String;

/// Maps to: CC `utils/ide.ts` `IDEExtensionInstallationStatus`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IDEExtensionInstallationStatus {
    pub installed: bool,
    pub error: Option<String>,
    pub installed_version: Option<String>,
    pub ide_type: Option<IdeType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IdeKind {
    VSCode,
    JetBrains,
}

fn supported_ide_config(ide: &str) -> Option<(IdeKind, &'static str)> {
    match ide {
        "cursor" => Some((IdeKind::VSCode, "Cursor")),
        "windsurf" => Some((IdeKind::VSCode, "Windsurf")),
        "vscode" => Some((IdeKind::VSCode, "VS Code")),
        "intellij" => Some((IdeKind::JetBrains, "IntelliJ IDEA")),
        "pycharm" => Some((IdeKind::JetBrains, "PyCharm")),
        "webstorm" => Some((IdeKind::JetBrains, "WebStorm")),
        "phpstorm" => Some((IdeKind::JetBrains, "PhpStorm")),
        "rubymine" => Some((IdeKind::JetBrains, "RubyMine")),
        "clion" => Some((IdeKind::JetBrains, "CLion")),
        "goland" => Some((IdeKind::JetBrains, "GoLand")),
        "rider" => Some((IdeKind::JetBrains, "Rider")),
        "datagrip" => Some((IdeKind::JetBrains, "DataGrip")),
        "appcode" => Some((IdeKind::JetBrains, "AppCode")),
        "dataspell" => Some((IdeKind::JetBrains, "DataSpell")),
        "aqua" => Some((IdeKind::JetBrains, "Aqua")),
        "gateway" => Some((IdeKind::JetBrains, "Gateway")),
        "fleet" => Some((IdeKind::JetBrains, "Fleet")),
        "androidstudio" => Some((IdeKind::JetBrains, "Android Studio")),
        _ => None,
    }
}

fn editor_display_name(command: &str) -> Option<&'static str> {
    match command {
        "code" => Some("VS Code"),
        "cursor" => Some("Cursor"),
        "windsurf" => Some("Windsurf"),
        "antigravity" => Some("Antigravity"),
        "vi" | "vim" => Some("Vim"),
        "nano" => Some("nano"),
        "notepad" | "start /wait notepad" => Some("Notepad"),
        "emacs" => Some("Emacs"),
        "subl" => Some("Sublime Text"),
        "atom" => Some("Atom"),
        _ => None,
    }
}

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}

/// Maps to: CC `utils/ide.ts` `isJetBrainsIde`.
pub fn is_jetbrains_ide(ide: Option<&str>) -> bool {
    ide.and_then(supported_ide_config)
        .map(|(kind, _)| kind == IdeKind::JetBrains)
        .unwrap_or(false)
}

/// Maps to: CC `utils/ide.ts` `isVSCodeIde`.
pub fn is_vscode_ide(ide: Option<&str>) -> bool {
    ide.and_then(supported_ide_config)
        .map(|(kind, _)| kind == IdeKind::VSCode)
        .unwrap_or(false)
}

/// Maps to: CC `utils/ide.ts` `toIDEDisplayName`.
pub fn to_ide_display_name(terminal: Option<&str>) -> String {
    let Some(terminal) = terminal.filter(|value| !value.is_empty()) else {
        return "IDE".to_string();
    };

    if let Some((_, display_name)) = supported_ide_config(terminal) {
        return display_name.to_string();
    }

    let trimmed_lower = terminal.to_lowercase();
    let trimmed_lower = trimmed_lower.trim();
    if let Some(display_name) = editor_display_name(trimmed_lower) {
        return display_name.to_string();
    }

    let command = terminal.split(' ').next().unwrap_or_default();
    let command_name = Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
        .to_lowercase();

    if !command_name.is_empty() {
        if let Some(display_name) = editor_display_name(&command_name) {
            return display_name.to_string();
        }
        return capitalize_ascii(&command_name);
    }

    capitalize_ascii(terminal)
}

/// Maps to: CC `components/IdeAutoConnectDialog.tsx`
/// `shouldShowAutoConnectDialog`, with `isSupportedTerminal()` supplied as an
/// already-known input.
pub fn should_show_auto_connect_dialog(config: &GlobalConfig, supported_terminal: bool) -> bool {
    !supported_terminal
        && config.auto_connect_ide != Some(true)
        && config.has_ide_auto_connect_dialog_been_shown != Some(true)
}

/// Maps to: CC `components/IdeAutoConnectDialog.tsx`
/// `shouldShowDisableAutoConnectDialog`, with `isSupportedTerminal()` supplied
/// as an already-known input.
pub fn should_show_disable_auto_connect_dialog(
    config: &GlobalConfig,
    supported_terminal: bool,
) -> bool {
    !supported_terminal && config.auto_connect_ide == Some(true)
}

/// Maps to: CC `components/IdeOnboardingDialog.tsx`
/// `hasIdeOnboardingDialogBeenShown`.
pub fn has_ide_onboarding_dialog_been_shown(config: &GlobalConfig, terminal: Option<&str>) -> bool {
    let terminal = terminal.unwrap_or("unknown");
    config
        .has_ide_onboarding_been_shown
        .as_ref()
        .and_then(|seen: &HashMap<String, bool>| seen.get(terminal))
        == Some(&true)
}

/// Maps to: CC `components/IdeOnboardingDialog.tsx` shortcut selection.
pub fn ide_mention_shortcut(platform: Option<&str>) -> &'static str {
    let platform = platform.unwrap_or(if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        "linux"
    });
    if platform == "darwin" {
        "Cmd+Option+K"
    } else {
        "Ctrl+Alt+K"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ide_display_name_matches_official_supported_and_editor_fallbacks() {
        assert_eq!(to_ide_display_name(None), "IDE");
        assert_eq!(to_ide_display_name(Some("vscode")), "VS Code");
        assert_eq!(to_ide_display_name(Some("intellij")), "IntelliJ IDEA");
        assert_eq!(to_ide_display_name(Some("/usr/bin/code --wait")), "VS Code");
        assert_eq!(to_ide_display_name(Some("vim")), "Vim");
        assert_eq!(
            to_ide_display_name(Some("custom-editor --wait")),
            "Custom-editor"
        );
    }

    #[test]
    fn ide_kind_helpers_match_official_config_table() {
        assert!(is_vscode_ide(Some("cursor")));
        assert!(is_vscode_ide(Some("vscode")));
        assert!(is_jetbrains_ide(Some("pycharm")));
        assert!(is_jetbrains_ide(Some("androidstudio")));
        assert!(!is_jetbrains_ide(Some("vscode")));
        assert!(!is_vscode_ide(None));
    }

    #[test]
    fn ide_auto_connect_dialog_gates_match_official_config_checks() {
        let mut config = GlobalConfig::default();
        assert!(should_show_auto_connect_dialog(&config, false));
        assert!(!should_show_auto_connect_dialog(&config, true));

        config.auto_connect_ide = Some(true);
        assert!(!should_show_auto_connect_dialog(&config, false));
        assert!(should_show_disable_auto_connect_dialog(&config, false));
        assert!(!should_show_disable_auto_connect_dialog(&config, true));

        config.auto_connect_ide = Some(false);
        config.has_ide_auto_connect_dialog_been_shown = Some(true);
        assert!(!should_show_auto_connect_dialog(&config, false));
    }

    #[test]
    fn ide_onboarding_seen_and_shortcut_match_official_helpers() {
        let mut config = GlobalConfig::default();
        assert!(!has_ide_onboarding_dialog_been_shown(
            &config,
            Some("vscode")
        ));
        config.has_ide_onboarding_been_shown = Some(HashMap::from([("vscode".to_string(), true)]));
        assert!(has_ide_onboarding_dialog_been_shown(
            &config,
            Some("vscode")
        ));
        assert!(!has_ide_onboarding_dialog_been_shown(
            &config,
            Some("cursor")
        ));
        assert_eq!(ide_mention_shortcut(Some("darwin")), "Cmd+Option+K");
        assert_eq!(ide_mention_shortcut(Some("linux")), "Ctrl+Alt+K");
    }
}
