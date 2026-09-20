//! Maps to: CC `utils/stringUtils.ts`.

/// Maps to: CC `utils/stringUtils.ts#capitalize` —
/// `str.charAt(0).toUpperCase() + str.slice(1)`.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Maps to: CC `utils/stringUtils.ts#plural`.
pub fn plural(n: usize, word: &str, plural_word: Option<&str>) -> String {
    if n == 1 {
        word.to_string()
    } else {
        plural_word.unwrap_or(&format!("{word}s")).to_string()
    }
}

/// Maps to CC `utils/stringUtils.ts#countCharInString` using JS UTF-16
/// code-unit offsets and `indexOf(..., previous + 1)` overlap semantics.
pub fn count_char_in_string(value: &str, needle: &str, start: usize) -> usize {
    let value = value.encode_utf16().collect::<Vec<_>>();
    let needle = needle.encode_utf16().collect::<Vec<_>>();
    if needle.is_empty() {
        return value.len().saturating_sub(start.min(value.len())) + 1;
    }
    let mut count = 0usize;
    let mut index = start.min(value.len());
    while index + needle.len() <= value.len() {
        if value[index..].starts_with(&needle) {
            count += 1;
            index += 1;
        } else {
            index += 1;
        }
    }
    count
}

/// Maps to: CC `utils/stringUtils.ts#normalizeFullWidthDigits`.
pub fn normalize_full_width_digits(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            '０'..='９' => char::from_u32(ch as u32 - 0xfee0).unwrap_or(ch),
            _ => ch,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_matches_official_default_and_custom_plural() {
        assert_eq!(plural(1, "file", None), "file");
        assert_eq!(plural(2, "file", None), "files");
        assert_eq!(plural(2, "entry", Some("entries")), "entries");
    }

    #[test]
    fn count_char_uses_utf16_start_offsets_and_overlaps() {
        assert_eq!(count_char_in_string("a\n😀\nb", "\n", 0), 2);
        assert_eq!(count_char_in_string("a\n😀\nb", "\n", 2), 1);
        assert_eq!(count_char_in_string("aaa", "aa", 0), 2);
    }

    #[test]
    fn normalize_full_width_digits_matches_official_mapping() {
        assert_eq!(
            normalize_full_width_digits("０１２３４５６７８９"),
            "0123456789"
        );
        assert_eq!(normalize_full_width_digits("a１b２"), "a1b2");
    }
}
