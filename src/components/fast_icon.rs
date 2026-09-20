//! Maps to: CC `components/FastIcon.tsx`.

use crate::constants::figures::LIGHTNING_BOLT;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct FastIconProps {
    pub cooldown: bool,
}

fn ansi_color(color: Color) -> String {
    match color {
        Color::Reset => "\x1b[39m".to_string(),
        Color::Black => "\x1b[38;5;0m".to_string(),
        Color::DarkRed => "\x1b[38;5;1m".to_string(),
        Color::DarkGreen => "\x1b[38;5;2m".to_string(),
        Color::DarkYellow => "\x1b[38;5;3m".to_string(),
        Color::DarkBlue => "\x1b[38;5;4m".to_string(),
        Color::DarkMagenta => "\x1b[38;5;5m".to_string(),
        Color::DarkCyan => "\x1b[38;5;6m".to_string(),
        Color::Grey => "\x1b[38;5;7m".to_string(),
        Color::DarkGrey => "\x1b[38;5;8m".to_string(),
        Color::Red => "\x1b[38;5;9m".to_string(),
        Color::Green => "\x1b[38;5;10m".to_string(),
        Color::Yellow => "\x1b[38;5;11m".to_string(),
        Color::Blue => "\x1b[38;5;12m".to_string(),
        Color::Magenta => "\x1b[38;5;13m".to_string(),
        Color::Cyan => "\x1b[38;5;14m".to_string(),
        Color::White => "\x1b[38;5;15m".to_string(),
        Color::AnsiValue(value) => format!("\x1b[38;5;{value}m"),
        Color::Rgb { r, g, b } => format!("\x1b[38;2;{r};{g};{b}m"),
    }
}

/// Maps to: CC `components/FastIcon.tsx#getFastIconString`.
pub fn get_fast_icon_string(
    apply_color: bool,
    cooldown: bool,
    theme: &crate::utils::theme::Theme,
) -> String {
    if !apply_color {
        return LIGHTNING_BOLT.to_string();
    }
    if cooldown {
        format!(
            "\x1b[2m{}{}\x1b[0m",
            ansi_color(theme.prompt_border),
            LIGHTNING_BOLT
        )
    } else {
        format!("{}{}\x1b[0m", ansi_color(theme.fast_mode), LIGHTNING_BOLT)
    }
}

/// Maps to: CC `components/FastIcon.tsx#FastIcon`.
#[component]
pub fn FastIcon(props: &FastIconProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let (color, dim) = if props.cooldown {
        (theme.prompt_border, true)
    } else {
        (theme.fast_mode, false)
    };

    element! {
        Text(content: LIGHTNING_BOLT.to_string(), color: color, dim: dim, wrap: TextWrap::NoWrap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    #[test]
    fn fast_icon_string_matches_plain_and_colored_branches() {
        let theme = *theme::current();
        assert_eq!(get_fast_icon_string(false, false, &theme), LIGHTNING_BOLT);
        assert!(get_fast_icon_string(true, false, &theme).contains(LIGHTNING_BOLT));
        assert!(get_fast_icon_string(true, true, &theme).starts_with("\x1b[2m"));
    }

    #[test]
    fn fast_icon_component_renders_fast_and_cooldown_colors() {
        let theme = *theme::current();
        let fast = element! {
            ContextProvider(value: Context::owned(theme)) { FastIcon(cooldown: false) }
        }
        .render(Some(20));
        assert_eq!(fast.to_string(), format!("{LIGHTNING_BOLT}\n"));
        assert_eq!(
            fast.resolved_text_style(0, 0).and_then(|style| style.color),
            Some(theme.fast_mode)
        );

        let cooldown = element! {
            ContextProvider(value: Context::owned(theme)) { FastIcon(cooldown: true) }
        }
        .render(Some(20));
        assert_eq!(
            cooldown
                .resolved_text_style(0, 0)
                .and_then(|style| style.color),
            Some(theme.prompt_border)
        );
    }
}
