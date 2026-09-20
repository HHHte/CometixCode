//! Anthropic API-enforced media limits.
//!
//! Maps to: CC `constants/apiLimits.ts:1-94`.

/// Maps to: CC `constants/apiLimits.ts:20` `API_IMAGE_MAX_BASE64_SIZE`.
pub const API_IMAGE_MAX_BASE64_SIZE: usize = 5 * 1024 * 1024;
/// Maps to: CC `constants/apiLimits.ts:27` `IMAGE_TARGET_RAW_SIZE`.
pub const IMAGE_TARGET_RAW_SIZE: usize = (API_IMAGE_MAX_BASE64_SIZE * 3) / 4;
/// Maps to: CC `constants/apiLimits.ts:40` `IMAGE_MAX_WIDTH`.
pub const IMAGE_MAX_WIDTH: u32 = 2_000;
/// Maps to: CC `constants/apiLimits.ts:41` `IMAGE_MAX_HEIGHT`.
pub const IMAGE_MAX_HEIGHT: u32 = 2_000;

/// Maps to: CC `constants/apiLimits.ts:53` `PDF_TARGET_RAW_SIZE`.
pub const PDF_TARGET_RAW_SIZE: u64 = 20 * 1024 * 1024;
/// Maps to: CC `constants/apiLimits.ts:58` `API_PDF_MAX_PAGES`.
pub const API_PDF_MAX_PAGES: u32 = 100;
/// Maps to: CC `constants/apiLimits.ts:66` `PDF_EXTRACT_SIZE_THRESHOLD`.
pub const PDF_EXTRACT_SIZE_THRESHOLD: u64 = 3 * 1024 * 1024;
/// Maps to: CC `constants/apiLimits.ts:72` `PDF_MAX_EXTRACT_SIZE`.
pub const PDF_MAX_EXTRACT_SIZE: u64 = 100 * 1024 * 1024;
/// Maps to: CC `constants/apiLimits.ts:77` `PDF_MAX_PAGES_PER_READ`.
pub const PDF_MAX_PAGES_PER_READ: u32 = 20;
/// Maps to: CC `constants/apiLimits.ts:83` `PDF_AT_MENTION_INLINE_THRESHOLD`.
pub const PDF_AT_MENTION_INLINE_THRESHOLD: u64 = 10;

/// Maps to: CC `constants/apiLimits.ts:92` `API_MAX_MEDIA_PER_REQUEST`.
pub const API_MAX_MEDIA_PER_REQUEST: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_constants_matches_official_api_limits_owner() {
        assert_eq!(PDF_TARGET_RAW_SIZE, 20 * 1024 * 1024);
        assert_eq!(API_PDF_MAX_PAGES, 100);
        assert_eq!(PDF_EXTRACT_SIZE_THRESHOLD, 3 * 1024 * 1024);
        assert_eq!(PDF_MAX_EXTRACT_SIZE, 100 * 1024 * 1024);
        assert_eq!(PDF_MAX_PAGES_PER_READ, 20);
        assert_eq!(PDF_AT_MENTION_INLINE_THRESHOLD, 10);
    }
}
