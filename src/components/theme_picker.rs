//! Maps to: CC `components/ThemePicker.tsx`.
//!
//! Official ThemePicker owns preview theme state, syntax toggle writes, and
//! select callbacks. This Rust component keeps the render boundary pure:
//! focus/selection/syntax state are props, and callers own event handling and
//! persistence.

use crate::components::custom_select::{Select, SelectLayout, SelectOptionData};
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderBackground, ToolRenderLine, ToolRenderTone,
};
use crate::components::structured_diff;
use crate::components::structured_diff::color_diff::get_syntax_theme;
use crate::types::message::StructuredDiffHunk;
use crate::utils::theme::{self, THEME_PICKER_ORDER, Theme, ThemeName};
use iocraft::{Color, prelude::*};

#[derive(Default, Props)]
pub struct ThemePickerProps {
    pub focused_index: usize,
    pub selected_value: Option<String>,
    pub show_intro_text: bool,
    pub help_text: Option<String>,
    pub show_help_text_below: bool,
    pub hide_esc_to_cancel: bool,
    pub skip_exit_handling: bool,
    pub active_theme_name: Option<ThemeName>,
    pub syntax_highlighting_disabled: bool,
    pub syntax_disabled_env_value: Option<String>,
    pub columns: usize,
    pub exit_pending: bool,
    pub exit_key_name: Option<String>,
    /// Maps to `feature('AUTO_THEME')` in the official option list.
    pub auto_theme_enabled: bool,
}

pub fn theme_picker_options(auto_theme_enabled: bool) -> Vec<SelectOptionData> {
    let mut options = Vec::new();
    if auto_theme_enabled {
        options.push(SelectOptionData {
            label: "Auto (match terminal)".to_string(),
            value: "auto".to_string(),
            ..SelectOptionData::default()
        });
    }
    options.extend(
        THEME_PICKER_ORDER
            .iter()
            .map(|theme_name| SelectOptionData {
                label: theme_name.display_label().to_string(),
                value: theme_name.setting_value().to_string(),
                ..SelectOptionData::default()
            }),
    );
    options
}

pub fn theme_picker_syntax_footer(
    active_theme_name: ThemeName,
    syntax_highlighting_disabled: bool,
    syntax_disabled_env_value: Option<&str>,
) -> String {
    if let Some(value) = syntax_disabled_env_value {
        return format!("Syntax highlighting disabled (via CLAUDE_CODE_SYNTAX_HIGHLIGHT={value})");
    }
    if syntax_highlighting_disabled {
        return "Syntax highlighting disabled (Ctrl+T to enable)".to_string();
    }

    let syntax_theme = get_syntax_theme(active_theme_name.setting_value());
    if let Some(source) = syntax_theme.source {
        format!(
            "Syntax theme: {} (from {source}) (Ctrl+T to disable)",
            syntax_theme.theme
        )
    } else {
        format!("Syntax theme: {} (Ctrl+T to disable)", syntax_theme.theme)
    }
}

fn theme_picker_demo_diff_lines(
    width: usize,
    theme_name: ThemeName,
    syntax_highlighting_enabled: bool,
) -> Vec<ToolRenderLine> {
    let hunk = StructuredDiffHunk {
        old_start: 1,
        new_start: 1,
        old_lines: 3,
        new_lines: 3,
        lines: vec![
            " function greet() {".to_string(),
            "-  console.log(\"Hello, World!\");".to_string(),
            "+  console.log(\"Hello, Claude!\");".to_string(),
            " }".to_string(),
        ],
    };

    if syntax_highlighting_enabled {
        structured_diff::render_hunks_with_syntax(
            &[hunk],
            false,
            width.max(1),
            structured_diff::SyntaxHighlightOptions {
                file_path: Some("demo.js".to_string()),
                first_line: Some("function greet() {".to_string()),
                theme: get_syntax_theme(theme_name.setting_value()).highlight_theme(),
                prefix_content: None,
            },
        )
    } else {
        structured_diff::render_hunks(&[hunk], false, width.max(1))
    }
}

fn tool_background_color(theme: Theme, background: ToolRenderBackground) -> Color {
    match background {
        ToolRenderBackground::DiffAdded => theme.diff_added,
        ToolRenderBackground::DiffRemoved => theme.diff_removed,
        ToolRenderBackground::DiffAddedWord => theme.diff_added_word,
        ToolRenderBackground::DiffRemovedWord => theme.diff_removed_word,
    }
}

fn tool_tone_color(theme: Theme, tone: ToolRenderTone) -> Option<Color> {
    match tone {
        ToolRenderTone::Normal => None,
        ToolRenderTone::Success => Some(theme.success),
        ToolRenderTone::Warning => Some(theme.warning),
        ToolRenderTone::Error => Some(theme.error),
        ToolRenderTone::Inactive => Some(theme.inactive),
    }
}

#[component]
pub fn ThemePicker(props: &ThemePickerProps) -> impl Into<AnyElement<'static>> {
    let active_theme_name = props.active_theme_name.unwrap_or(ThemeName::Dark);
    let preview_theme = *theme::get_theme(active_theme_name);
    let options = theme_picker_options(props.auto_theme_enabled);
    let focused_index = props.focused_index.min(options.len().saturating_sub(1));
    let syntax_footer = theme_picker_syntax_footer(
        active_theme_name,
        props.syntax_highlighting_disabled,
        props.syntax_disabled_env_value.as_deref(),
    );
    let diff_width = props.columns.saturating_sub(4).clamp(40, 100);
    let diff_lines = theme_picker_demo_diff_lines(
        diff_width,
        active_theme_name,
        props.syntax_disabled_env_value.is_none() && !props.syntax_highlighting_disabled,
    );
    let help_text = props.help_text.clone().unwrap_or_default();
    let show_footer_help = props.show_help_text_below && !help_text.is_empty();
    let show_cancel_footer = !props.hide_esc_to_cancel;
    let footer_text = if props.exit_pending && !props.skip_exit_handling {
        format!(
            "Press {} again to exit",
            props
                .exit_key_name
                .as_deref()
                .filter(|name| !name.is_empty())
                .unwrap_or("Ctrl-C")
        )
    } else {
        "Enter to select · Esc to cancel".to_string()
    };

    element! {
        ContextProvider(value: Context::owned(preview_theme)) {
            View(flex_direction: FlexDirection::Column, row_gap: 1u32) {
                View(flex_direction: FlexDirection::Column, row_gap: 1u32) {
                    #(if props.show_intro_text {
                        element! { Text(content: "Let's get started.".to_string()) }.into_any()
                    } else {
                        element! { Text(content: "Theme".to_string(), color: preview_theme.permission, weight: Weight::Bold) }.into_any()
                    })
                    View(flex_direction: FlexDirection::Column) {
                        Text(
                            content: "Choose the text style that looks best with your terminal".to_string(),
                            weight: Weight::Bold,
                        )
                        #(if !help_text.is_empty() && !props.show_help_text_below {
                            Some(element! { Text(content: help_text.clone(), color: preview_theme.inactive) })
                        } else {
                            None
                        })
                    }
                    Select(
                        is_disabled: false,
                        hide_indexes: false,
                        visible_option_count: options.len(),
                        options: options,
                        focused_index: focused_index,
                        selected_value: props.selected_value.clone(),
                        visible_from_index: 0usize,
                        layout: SelectLayout::Compact,
                    )
                }
                View(
                    flex_direction: FlexDirection::Column,
                    border_style: BorderStyle::Dashed,
                    border_top: true,
                    border_bottom: true,
                    border_left: false,
                    border_right: false,
                    border_color: preview_theme.subtle,
                ) {
                    Text(content: "demo.js".to_string(), color: preview_theme.inactive, wrap: TextWrap::NoWrap)
                    #(diff_lines.into_iter().map(|line| {
                        let line_color = tool_tone_color(preview_theme, line.tone);
                        let background_color = line.background.map(|background| tool_background_color(preview_theme, background));
                        let dim_text = line.dim;
                        if !line.segments.is_empty() {
                            element! {
                                View(flex_direction: FlexDirection::Row) {
                                    #(line.segments.into_iter().map(|segment| {
                                        let segment_background = segment
                                            .background
                                            .map(|background| tool_background_color(preview_theme, background))
                                            .or(background_color);
                                        let segment_color = segment.foreground.or(line_color);
                                        let segment_dim = dim_text || segment.dim;
                                        element! {
                                            Text(
                                                content: segment.text,
                                                color: segment_color,
                                                background_color: segment_background,
                                                dim: segment_dim,
                                                wrap: TextWrap::NoWrap,
                                            )
                                        }
                                    }))
                                }
                            }.into_any()
                        } else {
                            element! {
                                Text(
                                    content: line.text,
                                    color: line_color,
                                    background_color: background_color,
                                    dim: dim_text,
                                    wrap: TextWrap::NoWrap,
                                )
                            }.into_any()
                        }
                    }))
                }
                Text(content: format!(" {syntax_footer}"), color: preview_theme.inactive)
                #(if !props.show_intro_text {
                    Some(element! {
                        View(flex_direction: FlexDirection::Column) {
                            #(if show_footer_help {
                                Some(element! { Text(content: help_text.clone(), color: preview_theme.inactive) })
                            } else {
                                None
                            })
                            #(if show_cancel_footer {
                                Some(element! { Text(content: footer_text.clone(), color: preview_theme.inactive, italic: true) })
                            } else {
                                None
                            })
                        }
                    })
                } else {
                    None
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_picker(props: ThemePickerProps) -> String {
        element! { ThemePicker(
            focused_index: props.focused_index,
            selected_value: props.selected_value,
            show_intro_text: props.show_intro_text,
            help_text: props.help_text,
            show_help_text_below: props.show_help_text_below,
            hide_esc_to_cancel: props.hide_esc_to_cancel,
            skip_exit_handling: props.skip_exit_handling,
            active_theme_name: props.active_theme_name,
            syntax_highlighting_disabled: props.syntax_highlighting_disabled,
            syntax_disabled_env_value: props.syntax_disabled_env_value,
            columns: props.columns,
            exit_pending: props.exit_pending,
            exit_key_name: props.exit_key_name,
            auto_theme_enabled: props.auto_theme_enabled,
        ) }
        .render(Some(120))
        .to_string()
    }

    #[test]
    fn theme_picker_options_match_official_external_order() {
        let options = theme_picker_options(false);
        assert_eq!(
            options
                .iter()
                .map(|option| option.label.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Dark mode",
                "Light mode",
                "Dark mode (colorblind-friendly)",
                "Light mode (colorblind-friendly)",
                "Dark mode (ANSI colors only)",
                "Light mode (ANSI colors only)",
            ]
        );
        assert_eq!(theme_picker_options(true)[0].label, "Auto (match terminal)");
    }

    #[test]
    fn theme_picker_renders_official_title_preview_and_footer() {
        let text = render_picker(ThemePickerProps {
            selected_value: Some("dark".to_string()),
            active_theme_name: Some(ThemeName::Dark),
            columns: 100,
            ..ThemePickerProps::default()
        });
        assert!(text.contains("Theme"), "canvas=\n{text}");
        assert!(
            text.contains("Choose the text style that looks best with your terminal"),
            "canvas=\n{text}"
        );
        assert!(text.contains("demo.js"), "canvas=\n{text}");
        assert!(text.contains("Hello, World!"), "canvas=\n{text}");
        assert!(text.contains("Hello, Claude!"), "canvas=\n{text}");
        assert!(
            text.contains("Syntax theme: Monokai Extended (Ctrl+T to disable)"),
            "canvas=\n{text}"
        );
        assert!(
            text.contains("Enter to select · Esc to cancel"),
            "canvas=\n{text}"
        );
    }

    #[test]
    fn theme_picker_intro_mode_matches_onboarding_shape_without_footer() {
        let text = render_picker(ThemePickerProps {
            show_intro_text: true,
            help_text: Some("To change this later, run /theme".to_string()),
            hide_esc_to_cancel: true,
            skip_exit_handling: true,
            active_theme_name: Some(ThemeName::Light),
            columns: 100,
            ..ThemePickerProps::default()
        });
        assert!(text.contains("Let's get started."), "canvas=\n{text}");
        assert!(
            text.contains("To change this later, run /theme"),
            "canvas=\n{text}"
        );
        assert!(!text.contains("Enter to select"), "canvas=\n{text}");
        assert!(!text.contains("Theme\n"), "canvas=\n{text}");
    }

    #[test]
    fn theme_picker_syntax_footer_matches_disabled_branches() {
        assert_eq!(
            theme_picker_syntax_footer(ThemeName::Dark, false, Some("0")),
            "Syntax highlighting disabled (via CLAUDE_CODE_SYNTAX_HIGHLIGHT=0)"
        );
        assert_eq!(
            theme_picker_syntax_footer(ThemeName::Dark, true, None),
            "Syntax highlighting disabled (Ctrl+T to enable)"
        );
    }
}
