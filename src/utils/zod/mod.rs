//! Zod v4 runtime carrier.
//!
//! Maps to: zod/v4 (the dependency `../rebuild` consumes, not a CC source
//! file). Design contract: `docs/ZOD_RS_DESIGN.md`. This is the settled
//! "mini-zod" port — a value-level combinator tree, NOT a derive macro and NOT
//! type derivation. Rust has no `z.infer`, so the projection structs each tool
//! keeps (e.g. `FileReadInput`) are built from `safe_parse`'s `data` output.
//!
//! One definition yields three observable projections, and all three are
//! behaviour (design §裁决原则):
//!   1. `to_json_schema()` — the JSON Schema the model sees, key-for-key.
//!   2. `safe_parse()`'s error — the exact copy inside `<tool_use_error>`.
//!   3. `safe_parse()`'s `data` — strip / preprocess / default applied; this is
//!      what downstream consumes (`toolExecution.ts:684`).
//!
//! Deliberately out of scope (design §明确不做): type derivation, and the
//! post-hook JS-runtime coercion each tool's `from_args` does on hook-rewritten
//! input — that stays with the tool, it is not zod.
//!
//! API subset is capped at what CC actually uses; each combinator carries a
//! `Maps to: zod/v4 <API>` note. Extend only when a new CC usage appears.

mod error;
#[cfg(test)]
mod fixtures_test;
mod json_schema;
// Private like its siblings: the canonical entry is the `pub use` below, which
// is what every consumer imports. A batch briefly widened this to `pub mod` to
// unblock a cross-batch import of the module path; that importer moved to the
// re-export, so the narrower surface is restored.
mod number_to_string;
mod parse;
mod schema;

pub use error::{Issue, IssueCode, PathSegment, ZodError};
pub use json_schema::{JSON_SCHEMA_DRAFT, to_json_schema};
pub use number_to_string::javascript_number_to_string;
pub use parse::safe_parse;
pub use schema::{
    CatchUndefinedFn, CheckFn, CheckIssue, ObjectField, PreprocessFn, RefineFn, Schema,
    SuperRefineIssue, TransformFn, any, array, boolean, coerce_string, discriminated_union,
    enumeration, js_string, literal, number, object, partial_record, passthrough_object,
    preprocess, record, record_with_key, strict_object, string, union,
};

/// The JSON value a schema parses from and into. `safe_parse` returns the
/// transformed `data`, never the input it was given.
pub type Value = serde_json::Value;

/// Field list used while building object schemas, in DECLARATION order — zod
/// preserves the order fields are written (fixture: disc_union_ok's
/// `required: ["type","file"]` follows declaration, not sort order).
pub(crate) type Shape = Vec<(&'static str, Schema)>;
