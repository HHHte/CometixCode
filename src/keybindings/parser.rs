//! Maps to: CC `keybindings/parser.ts` — keystroke/chord string parsing and
//! canonical display formatting.

use super::types::{Chord, KeybindingBlock, ParsedBinding, ParsedKeystroke};

/// Maps to: CC `parseKeystroke("ctrl+shift+k")`. Modifier aliases:
/// ctrl/control; alt/opt/option; meta; cmd/command/super/win → super.
/// Key aliases: esc→escape, return→enter, space→" ", arrow glyphs.
pub fn parse_keystroke(input: &str) -> ParsedKeystroke {
    let mut keystroke = ParsedKeystroke::default();
    for part in input.split('+') {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => keystroke.ctrl = true,
            "alt" | "opt" | "option" => keystroke.alt = true,
            "shift" => keystroke.shift = true,
            "meta" => keystroke.meta = true,
            "cmd" | "command" | "super" | "win" => keystroke.super_key = true,
            "esc" => keystroke.key = "escape".into(),
            "return" => keystroke.key = "enter".into(),
            "space" => keystroke.key = " ".into(),
            "↑" => keystroke.key = "up".into(),
            "↓" => keystroke.key = "down".into(),
            "←" => keystroke.key = "left".into(),
            "→" => keystroke.key = "right".into(),
            _ => keystroke.key = lower,
        }
    }
    keystroke
}

/// Maps to: CC `parseChord("ctrl+k ctrl+s")`. A lone `" "` IS the space key
/// binding, not a separator.
pub fn parse_chord(input: &str) -> Chord {
    if input == " " {
        return vec![parse_keystroke("space")];
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return vec![parse_keystroke("")];
    }
    trimmed.split_whitespace().map(parse_keystroke).collect()
}

/// Maps to: CC `keystrokeToString` — canonical display representation.
pub fn keystroke_to_string(ks: &ParsedKeystroke) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if ks.ctrl {
        parts.push("ctrl");
    }
    if ks.alt {
        parts.push("alt");
    }
    if ks.shift {
        parts.push("shift");
    }
    if ks.meta {
        parts.push("meta");
    }
    if ks.super_key {
        parts.push("cmd");
    }
    let display = key_to_display_name(&ks.key);
    parts.push(&display);
    parts.join("+")
}

/// Maps to: CC `chordToString` — space-joined canonical steps. Used as the
/// resolver's chord-winners map key, so it must be stable and unambiguous.
pub fn chord_to_string(chord: &[ParsedKeystroke]) -> String {
    chord
        .iter()
        .map(keystroke_to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Maps to: CC `DisplayPlatform` in `keybindings/parser.ts`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayPlatform {
    Macos,
    Windows,
    #[default]
    Linux,
    Wsl,
    Unknown,
}

/// Maps to: CC `keystrokeToDisplayString(...)`.
pub fn keystroke_to_display_string(
    keystroke: &ParsedKeystroke,
    platform: DisplayPlatform,
) -> String {
    let mut parts = Vec::<String>::new();
    if keystroke.ctrl {
        parts.push("ctrl".to_string());
    }
    if keystroke.alt || keystroke.meta {
        parts.push(if platform == DisplayPlatform::Macos {
            "opt".to_string()
        } else {
            "alt".to_string()
        });
    }
    if keystroke.shift {
        parts.push("shift".to_string());
    }
    if keystroke.super_key {
        parts.push(if platform == DisplayPlatform::Macos {
            "cmd".to_string()
        } else {
            "super".to_string()
        });
    }
    parts.push(key_to_display_name(&keystroke.key));
    parts.join("+")
}

/// Maps to: CC `chordToDisplayString(...)`.
pub fn chord_to_display_string(chord: &[ParsedKeystroke], platform: DisplayPlatform) -> String {
    chord
        .iter()
        .map(|keystroke| keystroke_to_display_string(keystroke, platform))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Maps to: CC `parseBindings(...)`.
pub fn parse_bindings(blocks: &[KeybindingBlock]) -> Vec<ParsedBinding> {
    let mut bindings = Vec::new();
    for block in blocks {
        for (key, action) in &block.bindings {
            bindings.push(ParsedBinding {
                chord: parse_chord(key),
                action: action.clone(),
                context: block.context.clone(),
            });
        }
    }
    bindings
}

/// Maps to: CC `keyToDisplayName`.
fn key_to_display_name(key: &str) -> String {
    match key {
        "escape" => "Esc".into(),
        " " => "Space".into(),
        "tab" => "tab".into(),
        "enter" => "Enter".into(),
        "backspace" => "Backspace".into(),
        "delete" => "Delete".into(),
        "up" => "↑".into(),
        "down" => "↓".into(),
        "left" => "←".into(),
        "right" => "→".into(),
        "pageup" => "PageUp".into(),
        "pagedown" => "PageDown".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        other => other.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_keystroke_handles_modifier_and_key_aliases_like_cc() {
        let ks = parse_keystroke("ctrl+shift+K");
        assert!(ks.ctrl && ks.shift && !ks.alt && !ks.super_key);
        assert_eq!(ks.key, "k");

        assert!(parse_keystroke("opt+x").alt);
        assert!(parse_keystroke("option+x").alt);
        assert!(parse_keystroke("cmd+c").super_key);
        assert!(parse_keystroke("win+c").super_key);
        assert!(parse_keystroke("meta+m").meta);
        assert_eq!(parse_keystroke("esc").key, "escape");
        assert_eq!(parse_keystroke("return").key, "enter");
        assert_eq!(parse_keystroke("space").key, " ");
        assert_eq!(parse_keystroke("↑").key, "up");
    }

    #[test]
    fn parse_chord_splits_steps_and_keeps_lone_space_as_space_key() {
        let chord = parse_chord("ctrl+x ctrl+k");
        assert_eq!(chord.len(), 2);
        assert_eq!(chord[0].key, "x");
        assert_eq!(chord[1].key, "k");
        assert!(chord[0].ctrl && chord[1].ctrl);

        let space = parse_chord(" ");
        assert_eq!(space.len(), 1);
        assert_eq!(space[0].key, " ");
    }

    #[test]
    fn display_string_uses_platform_names_and_collapses_alt_meta() {
        let key = parse_keystroke("meta+cmd+shift+k");
        assert_eq!(
            keystroke_to_display_string(&key, DisplayPlatform::Macos),
            "opt+shift+cmd+k"
        );
        assert_eq!(
            keystroke_to_display_string(&key, DisplayPlatform::Linux),
            "alt+shift+super+k"
        );
        assert_eq!(
            chord_to_display_string(&parse_chord("alt+x cmd+y"), DisplayPlatform::Macos),
            "opt+x cmd+y"
        );
    }

    #[test]
    fn parse_bindings_flattens_blocks_in_source_order() {
        let blocks = vec![KeybindingBlock {
            context: super::super::types::ContextName::Chat,
            bindings: vec![
                ("enter".to_string(), Some("chat:submit".to_string())),
                ("ctrl+x".to_string(), None),
            ],
            include_in_template: true,
        }];
        let parsed = parse_bindings(&blocks);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].action.as_deref(), Some("chat:submit"));
        assert_eq!(parsed[1].action, None);
    }

    #[test]
    fn keystroke_to_string_matches_cc_canonical_order_and_display_names() {
        let ks = parse_keystroke("shift+ctrl+escape");
        assert_eq!(keystroke_to_string(&ks), "ctrl+shift+Esc");
        assert_eq!(
            chord_to_string(&parse_chord("ctrl+x ctrl+k")),
            "ctrl+x ctrl+k"
        );
    }
}
