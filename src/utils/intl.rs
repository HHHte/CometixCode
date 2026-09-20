//! Maps to: CC `utils/intl.ts:10-51` (the shared segmenter functions).
//! CC's cached Intl.Segmenter ≙ a cached locale/descriptor and a thread-local
//! installed-ICU template through the official rust_icu crate defaults.
//! ICU data versions can differ from Bun; parity is checked against its oracle.
//! Iterators and their UTF-16 buffers stay on their own thread; each segment
//! operation clones the template as JavaScriptCore IntlSegmenter::segment does.

use std::cell::RefCell;
use std::sync::OnceLock;

use rust_icu_sys::{UBreakIteratorType, UWordBreak};
use rust_icu_ubrk::UBreakIterator;

/// Native representation of Intl.SegmentData; indices are UTF-8 byte offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment<'a> {
    pub segment: &'a str,
    pub index: usize,
    pub is_word_like: Option<bool>,
}

/// Native runtime descriptor for Intl.Segmenter. Segmentation policy stays in ICU.
#[derive(Debug)]
pub struct Segmenter {
    granularity: UBreakIteratorType,
    locale: String,
}

thread_local! {
    static WORD_TEMPLATE: RefCell<Option<UBreakIterator>> = const { RefCell::new(None) };
    static GRAPHEME_TEMPLATE: RefCell<Option<UBreakIterator>> = const { RefCell::new(None) };
}

impl Segmenter {
    /// Native implementation of Intl.Segmenter.prototype.segment.
    pub fn segment<'a>(&self, text: &'a str) -> Vec<Segment<'a>> {
        let template = match self.granularity {
            UBreakIteratorType::UBRK_WORD => &WORD_TEMPLATE,
            _ => &GRAPHEME_TEMPLATE,
        };
        let mut iterator = template.with(|cell| {
            let mut template = cell.borrow_mut();
            template
                .get_or_insert_with(|| {
                    UBreakIterator::try_new(self.granularity, &self.locale, "")
                        .expect("Intl.Segmenter: ICU constructor failed")
                })
                .safe_clone()
                .expect("Intl.Segmenter: ICU iterator clone failed")
        });
        iterator
            .set_text(text)
            .expect("Intl.Segmenter: ICU text conversion failed");

        // ICU uses the same UTF-16 units as JS. This map changes representation,
        // not boundaries; ICU never returns a break inside a surrogate pair.
        let mut byte_offsets = Vec::with_capacity(text.encode_utf16().count() + 1);
        for (byte, character) in text.char_indices() {
            byte_offsets.extend(std::iter::repeat_n(byte, character.len_utf16()));
        }
        byte_offsets.push(text.len());
        let mut start = iterator.first() as usize;
        let mut segments = Vec::new();
        while let Some(end) = iterator.next() {
            let end = end as usize;
            let index = byte_offsets[start];
            let status = iterator.get_rule_status();
            segments.push(Segment {
                segment: &text[index..byte_offsets[end]],
                index,
                // JSC IntlSegmentDataObject::create classifies the ICU
                // NUMBER/LETTER/KANA/IDEO ranges; whitespace/emoji remain false.
                is_word_like: (self.granularity == UBreakIteratorType::UBRK_WORD).then_some(
                    status >= UWordBreak::UBRK_WORD_NUMBER as i32
                        && status < UWordBreak::UBRK_WORD_IDEO_LIMIT as i32,
                ),
            });
            start = end;
        }
        segments
    }
}

// Runtime locale adapter for Intl.Segmenter(undefined, ...). macOS reads the
// same JSCOnly process-locale source as Bun; no caller-specific language policy.
fn default_locale() -> &'static str {
    static LOCALE: OnceLock<String> = OnceLock::new();
    LOCALE.get_or_init(|| {
        #[cfg(target_os = "macos")]
        {
            process_default_locale()
        }
        #[cfg(not(target_os = "macos"))]
        {
            let locale = rust_icu_uloc::get_default();
            if matches!(locale.label(), "en_US_POSIX" | "c") {
                "en-US".to_string()
            } else {
                locale
                    .to_language_tag(false)
                    .expect("Intl.Segmenter: default locale conversion failed")
            }
        }
    })
}

/// Maps to: CC `utils/intl.ts:13-21` getGraphemeSegmenter.
pub fn get_grapheme_segmenter() -> &'static Segmenter {
    static SEGMENTER: OnceLock<Segmenter> = OnceLock::new();
    SEGMENTER.get_or_init(|| Segmenter {
        granularity: UBreakIteratorType::UBRK_CHARACTER,
        locale: default_locale().to_string(),
    })
}

/// Maps to: CC `utils/intl.ts:26-31` firstGrapheme.
pub fn first_grapheme(text: &str) -> &str {
    if text.is_empty() {
        return "";
    }
    get_grapheme_segmenter()
        .segment(text)
        .first()
        .map_or("", |segment| segment.segment)
}

/// Maps to: CC `utils/intl.ts:37-44` lastGrapheme.
pub fn last_grapheme(text: &str) -> &str {
    if text.is_empty() {
        return "";
    }
    get_grapheme_segmenter()
        .segment(text)
        .last()
        .map_or("", |segment| segment.segment)
}

/// Maps to: CC `utils/intl.ts:46-51` getWordSegmenter.
pub fn get_word_segmenter() -> &'static Segmenter {
    static SEGMENTER: OnceLock<Segmenter> = OnceLock::new();
    SEGMENTER.get_or_init(|| Segmenter {
        granularity: UBreakIteratorType::UBRK_WORD,
        locale: default_locale().to_string(),
    })
}

// Bun's JSCOnly build selects WTF/unix/LanguageUnix.cpp, including on
// macOS. It queries the process C locale, not ICU's environment-derived
// default and not Cocoa's preferred languages.
#[cfg(target_os = "macos")]
fn process_default_locale() -> String {
    // SAFETY: a null second argument only queries LC_CTYPE. Copy the
    // returned C string immediately; this function never mutates locale.
    let raw = unsafe { libc::setlocale(libc::LC_CTYPE, std::ptr::null()) };
    let locale = if raw.is_null() {
        String::new()
    } else {
        unsafe { std::ffi::CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned()
    };
    let language = locale.split(['.', '@']).next().unwrap_or("");
    if language.is_empty()
        || language.eq_ignore_ascii_case("C")
        || language.eq_ignore_ascii_case("POSIX")
    {
        "en-US".to_string()
    } else {
        language.replace('_', "-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn native_iterator_clone_retains_text_with_accepted_icu78_boundaries() {
        // JSC IntlSegmenter.cpp:117-127: a clone owns a text lifetime separate
        // from the cached template and from subsequent segment() calls.
        let mut original =
            UBreakIterator::try_new(UBreakIteratorType::UBRK_WORD, "en-US", "hello:world").unwrap();
        let mut cloned = original.safe_clone().unwrap();
        original.set_text("你好世界").unwrap();
        drop(original);
        assert_eq!(cloned.first(), 0);
        let mut ends = Vec::new();
        while let Some(end) = cloned.next() {
            ends.push(end);
        }
        // Keep checking buffer ownership independently of the accepted ICU rule
        // difference: Bun/ICU74 word ends [5, 6, 11], native ICU78 [11].
        let retained_word_ends = ends;
        cloned.set_text("A\0👩‍💻B").unwrap();
        assert_eq!(cloned.first(), 0);
        let mut ends = Vec::new();
        while let Some(end) = cloned.next() {
            ends.push(end);
        }
        assert_eq!(ends.last(), Some(&8));
        cloned.set_text("").unwrap();
        assert_eq!(cloned.first(), 0);
        assert_eq!(cloned.next(), None);
        // User-accepted ICU difference (2026-09-13); this is a fixed Rust
        // expectation, not an expectation computed by the iterator under test.
        assert_eq!(retained_word_ends, vec![11]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn default_locale_matches_jsc_with_accepted_icu78_word_boundaries() {
        // Bun's JSCOnly LanguageUnix.cpp:33-38 queries the process C locale.
        // A fresh process starts in C even when LC_ALL advertises another locale.
        let _guard = crate::utils::env_utils::EnvVarGuard::set("LC_ALL", "ja_JP.UTF-8");
        assert_eq!(process_default_locale(), "en-US");
        let segments = get_word_segmenter().segment("hello:world");
        // Accepted ICU78 colon rule: Bun/ICU74 yields hello / : / world.
        // Locale selection itself must still match JSC above.
        assert_eq!(
            segments
                .iter()
                .map(|part| (part.segment, part.index, part.is_word_like))
                .collect::<Vec<_>>(),
            vec![("hello:world", 0, Some(true))]
        );
    }

    #[test]
    fn cached_segmenters_have_independent_iteration_state_across_threads() {
        assert!(std::ptr::eq(get_word_segmenter(), get_word_segmenter()));
        assert!(std::ptr::eq(
            get_grapheme_segmenter(),
            get_grapheme_segmenter()
        ));
        let first = get_word_segmenter().segment("你好 世界");
        let other = std::thread::spawn(|| {
            get_word_segmenter()
                .segment("こんにちは世界")
                .into_iter()
                .map(|part| (part.segment.to_string(), part.index, part.is_word_like))
                .collect::<Vec<_>>()
        })
        .join()
        .unwrap();
        assert_eq!(first[0].segment, "你好");
        assert_eq!(first[2].index, 7);
        assert_eq!(
            other,
            vec![
                ("こんにちは".into(), 0, Some(true)),
                ("世界".into(), 15, Some(true))
            ]
        );
    }

    #[test]
    fn first_last_grapheme_use_original_empty_combining_and_emoji_contract() {
        assert_eq!(first_grapheme(""), "");
        assert_eq!(last_grapheme(""), "");
        assert_eq!(first_grapheme("e\u{301} x"), "e\u{301}");
        assert_eq!(last_grapheme("x👩‍💻"), "👩‍💻");
        assert_eq!(
            get_grapheme_segmenter()
                .segment("A👩‍💻B")
                .iter()
                .map(|s| s.index)
                .collect::<Vec<_>>(),
            vec![0, 1, 12]
        );
    }
    #[test]
    fn word_segments_match_bun_oracle_with_accepted_icu78_differences() {
        // Executed rebuild/src/utils/intl.ts on Bun/system ICU; every
        // segment (including non-words) is checked, not a script heuristic.
        // Accepted 2026-09-13: the colon case pins native ICU78 instead;
        // all other fixtures retain the actual Bun source result.
        let cases: &[(&str, &[(&str, usize, bool)])] = &[
            ("", &[]),
            (
                "你好 世界",
                &[("你好", 0, true), (" ", 6, false), ("世界", 7, true)],
            ),
            ("你好世界", &[("你好", 0, true), ("世界", 6, true)]),
            (
                "私は学生です",
                &[
                    ("私", 0, true),
                    ("は", 3, true),
                    ("学生", 6, true),
                    ("です", 12, true),
                ],
            ),
            (
                "こんにちは世界",
                &[("こんにちは", 0, true), ("世界", 15, true)],
            ),
            (
                "안녕하세요 세계",
                &[
                    ("안녕하세요", 0, true),
                    (" ", 15, false),
                    ("세계", 16, true),
                ],
            ),
            (
                "alpha-beta",
                &[("alpha", 0, true), ("-", 5, false), ("beta", 6, true)],
            ),
            // Bun/ICU74: [("hello", 0, true), (":", 5, false), ("world", 6, true)].
            // User-accepted native ICU78 word rule.
            ("hello:world", &[("hello:world", 0, true)]),
            (
                "can't foo_bar 3.14",
                &[
                    ("can't", 0, true),
                    (" ", 5, false),
                    ("foo_bar", 6, true),
                    (" ", 13, false),
                    ("3.14", 14, true),
                ],
            ),
            (
                "élan café",
                &[("élan", 0, true), (" ", 6, false), ("café", 7, true)],
            ),
            ("A👩‍💻B", &[("A", 0, true), ("👩‍💻", 1, false), ("B", 12, true)]),
            (
                "👨‍👩‍👧‍👦 hello",
                &[("👨‍👩‍👧‍👦", 0, false), (" ", 25, false), ("hello", 26, true)],
            ),
            (
                "👩‍💻你好，世界！",
                &[
                    ("👩‍💻", 0, false),
                    ("你好", 11, true),
                    ("，", 17, false),
                    ("世界", 20, true),
                    ("！", 26, false),
                ],
            ),
            (
                "[Image #12] x",
                &[
                    ("[", 0, false),
                    ("Image", 1, true),
                    (" ", 6, false),
                    ("#", 7, false),
                    ("12", 8, true),
                    ("]", 10, false),
                    (" ", 11, false),
                    ("x", 12, true),
                ],
            ),
            (
                "foo\nbar baz",
                &[
                    ("foo", 0, true),
                    ("\n", 3, false),
                    ("bar", 4, true),
                    (" ", 7, false),
                    ("baz", 8, true),
                ],
            ),
            (
                "ภาษาไทยภาษาไทย",
                &[
                    ("ภาษา", 0, true),
                    ("ไทย", 12, true),
                    ("ภาษา", 21, true),
                    ("ไทย", 33, true),
                ],
            ),
            ("កម្ពុជាភាសាខ្មែរ", &[("កម្ពុជា", 0, true), ("ភាសាខ្មែរ", 21, true)]),
            (
                "… — 👩‍💻",
                &[
                    ("…", 0, false),
                    (" ", 3, false),
                    ("—", 4, false),
                    (" ", 7, false),
                    ("👩‍💻", 8, false),
                ],
            ),
            (
                "中文abc def",
                &[
                    ("中文", 0, true),
                    ("abc", 6, true),
                    (" ", 9, false),
                    ("def", 10, true),
                ],
            ),
            ("กาแฟสวัสดี", &[("กาแฟ", 0, true), ("สวัสดี", 12, true)]),
        ];
        let mut differences = Vec::new();
        for (text, expected) in cases {
            let actual = get_word_segmenter()
                .segment(text)
                .into_iter()
                .map(|s| (s.segment, s.index, s.is_word_like.unwrap()))
                .collect::<Vec<_>>();
            if &actual != expected {
                differences.push(format!(
                    "text={text:?}: actual={actual:?}, Bun oracle / accepted ICU78 expectation={expected:?}"
                ));
            }
        }
        assert!(differences.is_empty(), "{}", differences.join("\n"));
    }
}
