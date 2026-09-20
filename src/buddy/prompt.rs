//! Maps to: CC `buddy/prompt.ts`.
//! Partial: `getCompanionIntroAttachment` and its companion-state producer are
//! not ported here; this owner supplies the source text for existing attachments.

/// Maps to: CC `buddy/prompt.ts:7-13` `companionIntroText`.
pub fn companion_intro_text(name: &str, species: &str) -> String {
    format!(
        "# Companion\n\nA small {species} named {name} sits beside the user's input box and occasionally comments in a speech bubble. You're not {name} — it's a separate watcher.\n\nWhen the user addresses {name} directly (by name), its bubble will answer. Your job in that moment is to stay out of the way: respond in ONE line or less, or just answer any part of the message meant for you. Don't explain that you're not {name} — they know. Don't narrate what {name} might say — the bubble handles that."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn companion_intro_text_matches_official_copy_and_interpolation() {
        assert_eq!(
            companion_intro_text("Pip", "fox"),
            "# Companion\n\nA small fox named Pip sits beside the user's input box and occasionally comments in a speech bubble. You're not Pip — it's a separate watcher.\n\nWhen the user addresses Pip directly (by name), its bubble will answer. Your job in that moment is to stay out of the way: respond in ONE line or less, or just answer any part of the message meant for you. Don't explain that you're not Pip — they know. Don't narrate what Pip might say — the bubble handles that."
        );
        let text = companion_intro_text("小狐{n}", "red\nfox");
        assert!(text.contains("A small red\nfox named 小狐{n} sits"));
        assert_eq!(text.matches("小狐{n}").count(), 5);
    }
}
