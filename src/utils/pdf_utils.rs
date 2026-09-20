//! PDF page-range helpers for the Read tool.
//!
//! Maps to: CC `utils/pdfUtils.ts:1-70`.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Maps to: CC `utils/pdfUtils.ts:4` `DOCUMENT_EXTENSIONS`.
static DOCUMENT_EXTENSIONS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["pdf"]));

/// Maps to: CC `utils/pdfUtils.ts:16-50` `parsePDFPageRange` result. Pages are 1-indexed; `None` preserves
/// JavaScript `Infinity` for an open-ended `N-` range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfPageRange {
    pub first_page: f64,
    /// `None` represents JavaScript `Infinity` for an open-ended `N-` range.
    pub last_page: Option<f64>,
}

fn javascript_parse_int_base10(value: &str) -> Option<f64> {
    let trimmed = value.trim_start_matches(|character| {
        matches!(
            character,
            '\u{0009}'..='\u{000d}'
                | '\u{0020}'
                | '\u{00a0}'
                | '\u{1680}'
                | '\u{2000}'..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
        )
    });
    let (negative, source) = match trimmed.as_bytes().first() {
        Some(b'+') => (false, &trimmed[1..]),
        Some(b'-') => (true, &trimmed[1..]),
        _ => (false, trimmed),
    };
    let digits = source
        .bytes()
        .take_while(u8::is_ascii_digit)
        .collect::<Vec<_>>();
    if digits.is_empty() || negative {
        return None;
    }
    std::str::from_utf8(&digits).ok()?.parse::<f64>().ok()
}

/// Maps to: CC `utils/pdfUtils.ts:16-50` `parsePDFPageRange(pages)`, including JavaScript
/// `parseInt(..., 10)` prefix parsing rather than Rust whole-string parsing.
pub fn parse_pdf_page_range(pages: &str) -> Option<PdfPageRange> {
    let trimmed = pages.trim_matches(|character| {
        matches!(
            character,
            '\u{0009}'..='\u{000d}'
                | '\u{0020}'
                | '\u{00a0}'
                | '\u{1680}'
                | '\u{2000}'..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
        )
    });
    if trimmed.is_empty() {
        return None;
    }

    if let Some(first_str) = trimmed.strip_suffix('-') {
        let first = javascript_parse_int_base10(first_str)?;
        if first < 1.0 {
            return None;
        }
        return Some(PdfPageRange {
            first_page: first,
            last_page: None,
        });
    }

    if let Some(dash_index) = trimmed.find('-') {
        let first = javascript_parse_int_base10(&trimmed[..dash_index])?;
        let last = javascript_parse_int_base10(&trimmed[dash_index + 1..])?;
        if first < 1.0 || last < 1.0 || last < first {
            return None;
        }
        return Some(PdfPageRange {
            first_page: first,
            last_page: Some(last),
        });
    }

    let page = javascript_parse_int_base10(trimmed)?;
    if page < 1.0 {
        return None;
    }
    Some(PdfPageRange {
        first_page: page,
        last_page: Some(page),
    })
}

/// Maps to: CC `utils/pdfUtils.ts:59-61` `isPDFSupported`.
pub fn is_pdf_supported() -> bool {
    !crate::utils::model::model::get_main_loop_model()
        .to_ascii_lowercase()
        .contains("claude-3-haiku")
}

/// Maps to: CC `utils/pdfUtils.ts:67-70` `isPDFExtension(ext)` — with or without leading dot.
pub fn is_pdf_extension(ext: &str) -> bool {
    let normalized = ext.strip_prefix('.').unwrap_or(ext).to_ascii_lowercase();
    DOCUMENT_EXTENSIONS.contains(normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pdf_page_range_matches_official_formats() {
        assert_eq!(
            parse_pdf_page_range("5"),
            Some(PdfPageRange {
                first_page: 5.0,
                last_page: Some(5.0)
            })
        );
        assert_eq!(
            parse_pdf_page_range("1-10"),
            Some(PdfPageRange {
                first_page: 1.0,
                last_page: Some(10.0)
            })
        );
        assert_eq!(
            parse_pdf_page_range("3-"),
            Some(PdfPageRange {
                first_page: 3.0,
                last_page: None
            })
        );
        assert_eq!(parse_pdf_page_range("0"), None);
        assert_eq!(parse_pdf_page_range("10-1"), None);
        assert_eq!(parse_pdf_page_range(""), None);
        assert_eq!(
            parse_pdf_page_range("1foo"),
            Some(PdfPageRange {
                first_page: 1.0,
                last_page: Some(1.0)
            })
        );
        assert_eq!(
            parse_pdf_page_range("4294967295"),
            Some(PdfPageRange {
                first_page: 4_294_967_295.0,
                last_page: Some(4_294_967_295.0)
            })
        );
        assert_eq!(
            parse_pdf_page_range("1-2-3"),
            Some(PdfPageRange {
                first_page: 1.0,
                last_page: Some(2.0)
            })
        );
        assert_eq!(
            parse_pdf_page_range("\u{feff} 2 \u{feff}"),
            Some(PdfPageRange {
                first_page: 2.0,
                last_page: Some(2.0)
            })
        );
        assert_eq!(parse_pdf_page_range("\u{0085}2"), None);
        assert_eq!(parse_pdf_page_range("\u{180e}2"), None);
    }

    #[test]
    fn is_pdf_extension_accepts_with_or_without_dot() {
        assert!(is_pdf_extension("pdf"));
        assert!(is_pdf_extension(".PDF"));
        assert!(!is_pdf_extension("png"));
    }
}
