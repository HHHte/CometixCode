//! Maps to: CC `utils/hash.ts`. Only the disk-stable djb2 family is ported here.

/// Maps to: CC `utils/hash.ts#djb2Hash:7-13`.
pub fn djb2_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn djb2_hash_matches_official_utf16_and_signed_overflow() {
        assert_eq!(djb2_hash(""), 0);
        assert_eq!(djb2_hash("abc"), 96354);
        assert_eq!(djb2_hash("😀"), 1772899);
        assert_eq!(djb2_hash("polygenelubricants"), i32::MIN);
        assert_eq!(djb2_hash("zzzzzz"), -685785664);
    }
}
