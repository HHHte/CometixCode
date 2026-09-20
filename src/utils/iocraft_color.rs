//! iocraft renderer-color adapters.
//!
//! Structural framework replacement for CC `utils/ink.ts`; see `PORTING.md`
//! under “React/Ink → iocraft”. Agent color ownership remains in
//! `tools/agent_tool/agent_color_manager.rs`.

use iocraft::Color;

use crate::tools::agent_tool::agent_color_manager::parse_agent_color_name;
use crate::utils::theme::Theme;

/// iocraft equivalent of CC `utils/ink.ts#toInkColor`.
///
/// Known agent colors resolve through the active theme. Other supported ANSI
/// names preserve CC's `ansi:<name>` fallback; unknown values leave terminal
/// color unchanged via `Color::Reset`.
pub fn to_iocraft_color(color: Option<&str>, theme: Theme) -> Color {
    let Some(color) = color else {
        return theme.agent_cyan;
    };
    if let Some(agent_color) = parse_agent_color_name(color) {
        return theme.color(agent_color.theme_key());
    }
    match color {
        "black" => Color::Black,
        "red" => Color::DarkRed,
        "green" => Color::DarkGreen,
        "yellow" => Color::DarkYellow,
        "blue" => Color::DarkBlue,
        "magenta" => Color::DarkMagenta,
        "cyan" => Color::DarkCyan,
        "white" => Color::Grey,
        "blackBright" => Color::DarkGrey,
        "redBright" => Color::Red,
        "greenBright" => Color::Green,
        "yellowBright" => Color::Yellow,
        "blueBright" => Color::Blue,
        "magentaBright" => Color::Magenta,
        "cyanBright" => Color::Cyan,
        "whiteBright" => Color::White,
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_iocraft_color_preserves_agent_theme_and_ansi_fallbacks() {
        let theme = *crate::utils::theme::current();
        assert_eq!(to_iocraft_color(None, theme), theme.agent_cyan);
        assert_eq!(to_iocraft_color(Some("purple"), theme), theme.agent_purple);
        assert_eq!(to_iocraft_color(Some("redBright"), theme), Color::Red);
        assert_eq!(to_iocraft_color(Some("unknown"), theme), Color::Reset);
    }
}
