//! Maps to: CC `components/design-system/ThemeProvider.tsx`.
//! The Rust main-screen path already passes a resolved `Theme` through context.
//! This module provides a readonly preview state seam and a provider wrapper so
//! future pickers can reuse the official save/preview/cancel vocabulary without
//! writing config by default.

use crate::utils::theme::{self, Theme, ThemeName};
use iocraft::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeSetting {
    Auto,
    Named(ThemeName),
}

impl Default for ThemeSetting {
    fn default() -> Self {
        Self::Named(ThemeName::Dark)
    }
}

pub fn resolve_theme_setting(setting: ThemeSetting, system_theme: ThemeName) -> ThemeName {
    match setting {
        ThemeSetting::Auto => system_theme,
        ThemeSetting::Named(name) => name,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePreviewState {
    pub saved: ThemeSetting,
    pub preview: Option<ThemeSetting>,
    pub system_theme: ThemeName,
}

impl Default for ThemePreviewState {
    fn default() -> Self {
        Self {
            saved: ThemeSetting::default(),
            preview: None,
            system_theme: ThemeName::Dark,
        }
    }
}

impl ThemePreviewState {
    pub fn current_theme_name(self) -> ThemeName {
        resolve_theme_setting(self.preview.unwrap_or(self.saved), self.system_theme)
    }

    pub fn set_preview(&mut self, setting: ThemeSetting) {
        self.preview = Some(setting);
    }

    pub fn save_preview(&mut self) -> Option<ThemeSetting> {
        let preview = self.preview.take()?;
        self.saved = preview;
        Some(preview)
    }

    pub fn cancel_preview(&mut self) {
        self.preview = None;
    }
}

#[derive(Default, Props)]
pub struct ThemeProviderProps {
    pub theme_name: Option<ThemeName>,
    pub children: Vec<AnyElement<'static>>,
}

#[component]
pub fn ThemeProvider(props: &mut ThemeProviderProps) -> impl Into<AnyElement<'static>> {
    let theme_value: Theme = *theme::get_theme(props.theme_name.unwrap_or(ThemeName::Dark));
    let children = props.children.drain(..).collect::<Vec<_>>();

    element! {
        ContextProvider(value: Context::owned(theme_value)) {
            #(children)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_preview_state_matches_official_preview_save_cancel_flow() {
        let mut state = ThemePreviewState::default();
        assert_eq!(state.current_theme_name(), ThemeName::Dark);

        state.set_preview(ThemeSetting::Named(ThemeName::Light));
        assert_eq!(state.current_theme_name(), ThemeName::Light);
        assert_eq!(
            state.save_preview(),
            Some(ThemeSetting::Named(ThemeName::Light))
        );
        assert_eq!(state.saved, ThemeSetting::Named(ThemeName::Light));

        state.set_preview(ThemeSetting::Named(ThemeName::DarkAnsi));
        state.cancel_preview();
        assert_eq!(state.current_theme_name(), ThemeName::Light);
    }

    #[test]
    fn theme_provider_supplies_resolved_theme_context() {
        let canvas = element! {
            ThemeProvider(theme_name: Some(ThemeName::Light)) {
                Text(content: "theme".to_string(), color: theme::LIGHT.text)
            }
        }
        .render(Some(20));

        assert_eq!(canvas.to_string().trim_end(), "theme");
    }
}
