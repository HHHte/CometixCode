//! UI-only `/theme` local command.
//! Maps to: CC `commands/theme/` + `components/ThemePicker.tsx` keyboard ownership.
//!
//! `ThemePicker` is a pure render boundary; this wrapper owns focus / Esc /
//! Enter. Selection is an in-memory preview (same phase as `/model`).

use crate::components::theme_picker::{ThemePicker, theme_picker_options};
use crate::utils::theme::ThemeName;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ThemePickerWrapperProps<'a> {
    pub on_close: HandlerMut<'a, ()>,
    pub on_select: HandlerMut<'a, String>,
}

#[component]
pub fn ThemePickerWrapper<'a>(
    props: &mut ThemePickerWrapperProps<'a>,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let (columns, _) = hooks.use_terminal_size();
    let options = theme_picker_options(false);
    let option_count = options.len();
    let dark_value = ThemeName::Dark.setting_value();
    let initial_focus = options
        .iter()
        .position(|option| option.value == dark_value)
        .unwrap_or(0);

    let mut focused_index = hooks.use_state(|| initial_focus);
    let mut should_close = hooks.use_state(|| false);
    let mut pending_selection = hooks.use_state(|| Option::<String>::None);
    let event_options = options.clone();

    hooks.use_terminal_events({
        let event_options = event_options.clone();
        move |event| {
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                let focused = focused_index.get().min(option_count.saturating_sub(1));
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        focused_index.set(if focused == 0 {
                            option_count.saturating_sub(1)
                        } else {
                            focused - 1
                        });
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        focused_index.set(if option_count == 0 {
                            0
                        } else {
                            (focused + 1) % option_count
                        });
                    }
                    KeyCode::Enter => {
                        if let Some(option) = event_options.get(focused) {
                            pending_selection.set(Some(option.label.clone()));
                        }
                    }
                    KeyCode::Esc => {
                        should_close.set(true);
                    }
                    _ => {}
                }
            }
        }
    });

    if should_close.get() {
        should_close.set(false);
        (props.on_close)(());
    }
    let selected = { pending_selection.read().clone() };
    if let Some(label) = selected {
        pending_selection.set(None);
        (props.on_select)(format!(
            "Set theme to {label} (UI-only preview; theme was not persisted)"
        ));
    }

    let focused = focused_index.get().min(option_count.saturating_sub(1));
    let selected_value = options.get(focused).map(|option| option.value.clone());
    let active_theme_name = options
        .get(focused)
        .and_then(|option| ThemeName::from_setting_value(&option.value));

    element! {
        ThemePicker(
            focused_index: focused,
            selected_value: selected_value,
            show_intro_text: true,
            help_text: Some("Enter to select · Esc to cancel".to_string()),
            show_help_text_below: true,
            hide_esc_to_cancel: false,
            skip_exit_handling: true,
            active_theme_name: active_theme_name,
            syntax_highlighting_disabled: false,
            syntax_disabled_env_value: None,
            columns: usize::from(columns),
            exit_pending: false,
            exit_key_name: None,
            auto_theme_enabled: false,
        )
    }
}

pub fn handle_cancel_output() -> String {
    "Theme dialog dismissed".to_string()
}
