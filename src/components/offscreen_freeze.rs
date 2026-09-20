//! Maps to: CC `components/OffscreenFreeze.tsx`.
//!
//! Claude Code's React component freezes a subtree once native terminal
//! scrollback has moved it above the live viewport, and bypasses freezing when
//! `InVirtualListContext` is active. Cometix uses the equivalent iocraft
//! retained-render implementation at the same component boundary.

pub use iocraft::components::{InVirtualListContext, OffscreenFreeze, OffscreenFreezeProps};

#[cfg(test)]
mod tests {
    use super::*;
    use iocraft::prelude::*;

    #[test]
    fn offscreen_freeze_boundary_renders_children() {
        let text = element! {
            OffscreenFreeze(terminal_rows: Some(24u16)) {
                Text(content: "visible child".to_string())
            }
        }
        .render(Some(40))
        .to_string();

        assert!(text.contains("visible child"), "canvas=\n{text}");
    }

    #[test]
    fn offscreen_freeze_exports_virtual_list_context_marker() {
        fn accepts_marker(_: InVirtualListContext) {}
        accepts_marker(InVirtualListContext);
    }
}
