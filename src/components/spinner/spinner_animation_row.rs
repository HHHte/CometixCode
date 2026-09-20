//! Maps to: CC `components/Spinner/SpinnerAnimationRow.tsx`.
//!
//! UNFAITHFUL (ruled 2026-08-07, see MODULE_MAP): this file is an alias shell
//! while both entities live merged in `spinner/mod.rs`. CC splits the
//! container (`Spinner.tsx`, off-clock prop computation) from the animation
//! row, which owns the 50ms clock state (counter, elapsed-time, stalled
//! intensity, thinking shimmer). Pending split: make this a real component
//! owning that state; the "shared retained animation owner" rationale does
//! not hold — CC keeps that state in the row as well.

pub use super::{
    SpinnerWithVerb as SpinnerAnimationRow, SpinnerWithVerbProps as SpinnerAnimationRowProps,
};
