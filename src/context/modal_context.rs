//! Maps to: CC `context/modalContext.tsx`.
//!
//! Split out of `context.rs` in the 2026-08-04 naming batch: that file maps CC
//! `src/context.ts` (session/CLAUDE.md/git context), which is a different
//! source file that merely shares a directory name. One Rust file per CC file.

use iocraft::hooks::UseContext;

/// Maps to: CC `context/modalContext.tsx` `ModalContext`.
///
/// Provided by `FullscreenLayout` when rendering the modal slot. The scrollRef
/// field is transported alongside this value through ModalScrollRefContext,
/// preserving the existing Copy/Eq size snapshot used by layout consumers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModalContextSnapshot {
    pub rows: u16,
    pub columns: u16,
}

impl ModalContextSnapshot {
    /// Maps to: CC `context/modalContext.tsx#useModalOrTerminalSize`.
    pub fn size_or(self, fallback: (u16, u16)) -> (u16, u16) {
        let rows = if self.rows == 0 {
            fallback.1
        } else {
            self.rows
        };
        let columns = if self.columns == 0 {
            fallback.0
        } else {
            self.columns
        };
        (columns, rows)
    }
}

/// Rust context carrier for CC `context/modalContext.tsx:27#ModalCtx.scrollRef`.
/// Kept next to the existing size snapshot because a live ref is not a size value.
#[derive(Clone, Copy, Default)]
pub struct ModalScrollRefContext(
    pub Option<iocraft::prelude::Ref<iocraft::prelude::ScrollBoxHandle>>,
);

/// Maps to: CC `context/modalContext.tsx:32-34#useIsInsideModal`.
pub fn use_is_inside_modal(hooks: &iocraft::prelude::Hooks) -> bool {
    hooks.try_use_context::<ModalContextSnapshot>().is_some()
}

/// Maps to: CC `context/modalContext.tsx:55-57#useModalScrollRef`.
pub fn use_modal_scroll_ref(
    hooks: &iocraft::prelude::Hooks,
) -> Option<iocraft::prelude::Ref<iocraft::prelude::ScrollBoxHandle>> {
    hooks
        .try_use_context::<ModalScrollRefContext>()
        .and_then(|context| context.0)
}
