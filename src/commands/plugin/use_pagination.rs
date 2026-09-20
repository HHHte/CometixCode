//! Maps to: CC `commands/plugin/usePagination.ts`.
//!
//! L1: React refs/memo use iocraft refs/memo; returned callbacks are methods on
//! the render's result value. Slice views preserve element order and borrowing,
//! not JavaScript's fresh-array identity for `items.slice()`. Actual callers use
//! nonnegative item counts/indexes and positive page sizes: `usize` and
//! `NonZeroUsize` deliberately do not represent fractional/NaN/negative counts
//! or zero page sizes. Selection callbacks still accept negative candidate
//! indexes, which the source explicitly clamps. No plugin UI is wired here.

use iocraft::prelude::*;
use std::num::NonZeroUsize;

/// Maps to: CC `DEFAULT_MAX_VISIBLE`.
const DEFAULT_MAX_VISIBLE: usize = 5;

/// Maps to: CC `UsePaginationOptions`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UsePaginationOptions {
    pub total_items: usize,
    pub max_visible: Option<NonZeroUsize>,
    pub selected_index: Option<usize>,
}

/// Maps to: CC `UsePaginationResult.scrollPosition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollPosition {
    pub current: usize,
    pub total: usize,
    pub can_scroll_up: bool,
    pub can_scroll_down: bool,
}

/// Maps to: CC `handlePageNavigation`'s direction union.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageDirection {
    Left,
    Right,
}

/// Maps to: CC `UsePaginationResult<T>`; callback captures live in this render
/// snapshot, and generic item access remains on `get_visible_items`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsePaginationResult {
    pub current_page: usize,
    pub total_pages: usize,
    pub start_index: usize,
    pub end_index: usize,
    pub needs_pagination: bool,
    pub page_size: usize,
    pub scroll_position: ScrollPosition,
}

impl UsePaginationResult {
    /// Maps to: CC `getVisibleItems` (:89-95). With no pagination CC returns
    /// the supplied items unchanged, even if their length differs from totalItems.
    pub fn get_visible_items<'a, T>(&self, items: &'a [T]) -> &'a [T] {
        if !self.needs_pagination {
            return items;
        }
        let start = self.start_index.min(items.len());
        let end = self.end_index.min(items.len());
        &items[start..end.max(start)]
    }

    /// Maps to: CC `toActualIndex` (:97-102).
    pub fn to_actual_index(&self, visible_index: usize) -> usize {
        self.start_index + visible_index
    }

    /// Maps to: CC `isOnCurrentPage` (:104-109).
    pub fn is_on_current_page(&self, actual_index: usize) -> bool {
        actual_index >= self.start_index && actual_index < self.end_index
    }

    /// Maps to: CC `goToPage` (:112-114); intentionally does nothing.
    pub fn go_to_page(&self, _page: isize) {}

    /// Maps to: CC `nextPage` (:116-118); intentionally does nothing.
    pub fn next_page(&self) {}

    /// Maps to: CC `prevPage` (:120-122); intentionally does nothing.
    pub fn prev_page(&self) {}

    /// Maps to: CC `handleSelectionChange` (:126-132). Callback fires even if
    /// the clamped index equals the current selection, including empty lists.
    pub fn handle_selection_change(
        &self,
        new_index: isize,
        mut set_selected_index: impl FnMut(usize),
    ) {
        set_selected_index(
            (new_index.max(0) as usize).min(self.scroll_position.total.saturating_sub(1)),
        );
    }

    /// Maps to: CC `handlePageNavigation` (:135-143); no callback is invoked.
    pub fn handle_page_navigation(
        &self,
        _direction: PageDirection,
        _set_selected_index: impl FnMut(usize),
    ) -> bool {
        false
    }
}

/// Maps to: CC `usePagination` (:48-171).
pub fn use_pagination(hooks: &mut Hooks, options: UsePaginationOptions) -> UsePaginationResult {
    let total_items = options.total_items;
    let max_visible = options
        .max_visible
        .map_or(DEFAULT_MAX_VISIBLE, NonZeroUsize::get);
    let selected_index = options.selected_index.unwrap_or(0);
    let needs_pagination = total_items > max_visible;
    let mut scroll_offset_ref = hooks.use_ref(|| 0usize);
    let scroll_offset = hooks.use_memo(
        move || {
            // CC returns zero without overwriting the retained ref. A later
            // expansion can restore the previous visible window.
            if !needs_pagination {
                return 0;
            }
            let prev_offset = *scroll_offset_ref.read();
            if selected_index < prev_offset {
                *scroll_offset_ref.write() = selected_index;
                return selected_index;
            }
            if selected_index >= prev_offset + max_visible {
                let new_offset = selected_index - max_visible + 1;
                *scroll_offset_ref.write() = new_offset;
                return new_offset;
            }
            let max_offset = total_items.saturating_sub(max_visible);
            let clamped_offset = prev_offset.min(max_offset);
            *scroll_offset_ref.write() = clamped_offset;
            clamped_offset
        },
        (selected_index, max_visible, needs_pagination, total_items),
    );
    UsePaginationResult {
        current_page: scroll_offset / max_visible,
        total_pages: total_items.div_ceil(max_visible).max(1),
        start_index: scroll_offset,
        end_index: (scroll_offset + max_visible).min(total_items),
        needs_pagination,
        page_size: max_visible,
        scroll_position: ScrollPosition {
            current: selected_index + 1,
            total: total_items,
            can_scroll_up: scroll_offset > 0,
            can_scroll_down: scroll_offset + max_visible < total_items,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{StreamExt, stream};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    // Actual Bun + original React/Ink/usePagination oracle, 2026-09-13:
    // research/proof/plugin-pagination-0913/{oracle.ts,bun-oracle.json}.
    const OPTIONS: [(usize, usize, usize); 17] = [
        (0, 5, 0),
        (20, 5, 0),
        (20, 5, 8),
        (20, 5, 5),
        (3, 5, 0),
        (20, 5, 5),
        (20, 5, 3),
        (20, 5, 14),
        (7, 5, 6),
        (20, 3, 6),
        (20, 3, 6),
        (2, 3, 1),
        (20, 3, 6),
        (7, 3, 99),
        (20, 8, 19),
        (0, 8, 0),
        (20, 8, 19),
    ];
    const WINDOWS: [(usize, usize, usize, usize); 17] = [
        (0, 0, 0, 1),
        (0, 5, 0, 4),
        (4, 9, 0, 4),
        (4, 9, 0, 4),
        (0, 3, 0, 1),
        (4, 9, 0, 4),
        (3, 8, 0, 4),
        (10, 15, 2, 4),
        (6, 7, 1, 2),
        (6, 9, 2, 7),
        (6, 9, 2, 7),
        (0, 2, 0, 1),
        (6, 9, 2, 7),
        (97, 7, 32, 3),
        (19, 20, 2, 3),
        (0, 0, 0, 1),
        (12, 20, 1, 3),
    ];

    const VISIBLE: [&[usize]; 17] = [
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ],
        &[0, 1, 2, 3, 4],
        &[4, 5, 6, 7, 8],
        &[4, 5, 6, 7, 8],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ],
        &[4, 5, 6, 7, 8],
        &[3, 4, 5, 6, 7],
        &[10, 11, 12, 13, 14],
        &[6],
        &[6, 7, 8],
        &[6, 7, 8],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ],
        &[6, 7, 8],
        &[],
        &[19],
        &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ],
        &[12, 13, 14, 15, 16, 17, 18, 19],
    ];

    #[derive(Default, Props)]
    struct PaginationProbeProps {
        results: Arc<Mutex<Vec<UsePaginationResult>>>,
    }

    #[component]
    fn PaginationProbe(
        props: &PaginationProbeProps,
        mut hooks: Hooks,
    ) -> impl Into<AnyElement<'static>> {
        let mut step = hooks.use_state(|| 0usize);
        hooks.use_terminal_events(move |event| {
            if matches!(event, TerminalEvent::Key(key) if key.code == KeyCode::Enter) {
                step.set((step.get() + 1).min(OPTIONS.len() - 1));
            }
        });
        let index = step.get();
        let (total_items, max_visible, selected_index) = OPTIONS[index];
        let result = use_pagination(
            &mut hooks,
            UsePaginationOptions {
                total_items,
                max_visible: if max_visible == 5 {
                    None
                } else {
                    NonZeroUsize::new(max_visible)
                },
                selected_index: if index == 0 {
                    None
                } else {
                    Some(selected_index)
                },
            },
        );
        let mut results = props.results.lock().unwrap();
        if results.len() == index {
            results.push(result);
        }
        element! { Text(content: format!("step={index}")) }
    }

    #[test]
    fn pagination_retained_ref_memo_and_all_results_match_official_bun() {
        let results = Arc::new(Mutex::new(Vec::new()));
        futures::executor::block_on(async {
            let mut app = element! { PaginationProbe(results: results.clone()) };
            let events = stream::iter(1..OPTIONS.len())
                .then(|_| async {
                    futures_timer::Delay::new(Duration::from_millis(10)).await;
                    TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Enter))
                })
                .chain(stream::pending());
            let mut frames = Box::pin(app.mock_terminal_render_loop(
                MockTerminalConfig::with_events(events).with_size(40, 3),
            ));
            loop {
                let frame = crate::utils::race(frames.next(), async {
                    futures_timer::Delay::new(Duration::from_secs(2)).await;
                    None
                })
                .await;
                assert!(
                    frame.is_some(),
                    "pagination fixture did not advance through all renders"
                );
                if results.lock().unwrap().len() == OPTIONS.len() {
                    break;
                }
            }
        });
        let results = results.lock().unwrap();
        assert_eq!(results.len(), OPTIONS.len());
        let items: Vec<_> = (0..20).collect();
        for (i, result) in results.iter().enumerate() {
            let (total, size, selected) = OPTIONS[i];
            let (start, end, current_page, total_pages) = WINDOWS[i];
            // CC usePagination.ts:56-87,146-147: ref retention, early returns,
            // memo dependencies, window and legacy page projection; actual Bun WINDOWS.
            assert_eq!(
                (
                    result.start_index,
                    result.end_index,
                    result.current_page,
                    result.total_pages
                ),
                (start, end, current_page, total_pages),
                "oracle render {i}"
            );
            // CC usePagination.ts:50-53,155: default size and strict pagination threshold.
            assert_eq!(result.page_size, size);
            assert_eq!(result.needs_pagination, total > size);
            // CC usePagination.ts:164-168: empty current remains1 and scroll flags use offset.
            assert_eq!(
                result.scroll_position,
                ScrollPosition {
                    current: selected + 1,
                    total,
                    can_scroll_up: start > 0,
                    can_scroll_down: start + size < total,
                }
            );
            // CC usePagination.ts:89-95: expected lists come directly from Bun oracle.
            let visible = result.get_visible_items(&items);
            assert_eq!(visible, VISIBLE[i], "visible items at oracle render {i}");
            if total <= size {
                assert!(std::ptr::eq(visible, items.as_slice()));
            }
            // CC usePagination.ts:97-109: conversions and half-open membership.
            for visible_index in [0, 2] {
                assert_eq!(result.to_actual_index(visible_index), start + visible_index);
            }
            for actual in [0, 3, 4, 5, 8, 9, 10, 14] {
                assert_eq!(
                    result.is_on_current_page(actual),
                    actual >= start && actual < end
                );
            }
            // CC usePagination.ts:112-143: page methods do not mutate/call selection;
            // selection invokes callback even for unchanged values and empty lists.
            let mut calls = Vec::new();
            let unchanged = *result;
            result.go_to_page(9);
            result.next_page();
            result.prev_page();
            for direction in [PageDirection::Left, PageDirection::Right] {
                assert!(!result.handle_page_navigation(direction, |value| calls.push(value)));
            }
            assert!(calls.is_empty());
            assert_eq!(*result, unchanged);
            for candidate in [-9, 0, 3, 99] {
                result.handle_selection_change(candidate, |value| calls.push(value));
            }
            let expected = match total {
                0 => [0, 0, 0, 0],
                2 => [0, 0, 1, 1],
                3 => [0, 0, 2, 2],
                7 => [0, 0, 3, 6],
                20 => [0, 0, 3, 19],
                _ => unreachable!(),
            };
            assert_eq!(calls, expected);
        }
        // CC usePagination.ts:92: JS slice clamps even when items is shorter than totalItems.
        assert!(results[7].get_visible_items(&[1, 2]).is_empty());
    }

    #[derive(Default, Props)]
    struct PaginationMountProbeProps {
        selected_index: usize,
        result: Arc<Mutex<Option<UsePaginationResult>>>,
    }

    #[component]
    fn PaginationMountProbe(
        props: &PaginationMountProbeProps,
        mut hooks: Hooks,
    ) -> impl Into<AnyElement<'static>> {
        let result = use_pagination(
            &mut hooks,
            UsePaginationOptions {
                total_items: 20,
                selected_index: Some(props.selected_index),
                ..Default::default()
            },
        );
        *props.result.lock().unwrap() = Some(result);
        element! { View }
    }

    #[test]
    fn pagination_unmount_remount_resets_ref_matches_official_bun() {
        // CC usePagination.ts:56,72-75: each mount owns a fresh useRef(0).
        // Actual original Ink root unmount/createRoot produced start1/end6 for
        // selected5 (remount-bun-oracle.json), rather than retaining offset4.
        for (selected_index, expected) in [(8usize, (4, 9)), (5usize, (1, 6))] {
            let result = Arc::new(Mutex::new(None));
            {
                let mut app =
                    element! { PaginationMountProbe(selected_index, result: result.clone()) };
                app.render(None);
            }
            let result = result.lock().unwrap().unwrap();
            assert_eq!((result.start_index, result.end_index), expected);
        }
    }
}
