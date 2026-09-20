//! Maps to: CC `components/StructuredDiff/Fallback.tsx`.
//! The fallback selects the shared native line/word diff engine without syntax
//! highlighting, preserving a distinct component boundary.

use crate::components::structured_diff::StructuredDiff;
use crate::types::message::StructuredDiffHunk;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct StructuredDiffFallbackProps {
    pub patch: StructuredDiffHunk,
    pub dim: bool,
    pub width: usize,
}

#[component]
pub fn StructuredDiffFallback(
    props: &StructuredDiffFallbackProps,
) -> impl Into<AnyElement<'static>> {
    element!(StructuredDiff(
        patch: props.patch.clone(),
        dim: props.dim,
        width: props.width,
        skip_highlighting: true,
    ))
}
