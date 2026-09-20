//! The error a failed `safe_parse` yields.
//!
//! Maps to: zod/v4 `ZodError` / its `issues`. Two things are load-bearing and
//! easy to get wrong from memory:
//!
//! 1. `message` is the whole issues tree as `JSON.stringify(issues, null, 2)`,
//!    NOT a single issue's sentence. It is the fallback `errorContent` in
//!    `formatZodValidationError` (`toolErrors.ts:99`) for every code that
//!    function does not rebuild, and it lands in `toolUseResult`
//!    (`toolExecution.ts:672`). So the dump must be byte-faithful.
//! 2. Each issue carries the fields zod puts on it, in zod's property order,
//!    because the dump's bytes depend on both.

use serde_json::{Map, Value, json};

/// Maps to: zod/v4 issue `code`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IssueCode {
    /// Maps to: zod/v4 `invalid_type`.
    #[default]
    InvalidType,
    /// Maps to: zod/v4 `unrecognized_keys` (strictObject stray keys).
    UnrecognizedKeys,
    /// Maps to: zod/v4 `invalid_union` (also discriminated-union failures).
    InvalidUnion,
    /// Maps to: zod/v4 `too_small` / `too_big`.
    TooSmall,
    TooBig,
    /// Maps to: zod/v4 `invalid_value` (literal / enum mismatch).
    InvalidValue,
    /// Maps to: zod/v4 `invalid_format` (a `.regex` / `.url` failure).
    InvalidFormat,
    /// Maps to: zod/v4 `invalid_key` (a `partialRecord` key outside its enum).
    InvalidKey,
    /// Maps to: zod/v4 `custom` (a `.refine` failure).
    Custom,
}

impl IssueCode {
    /// Maps to: zod/v4's issue `code` string.
    pub fn as_str(self) -> &'static str {
        match self {
            IssueCode::InvalidType => "invalid_type",
            IssueCode::UnrecognizedKeys => "unrecognized_keys",
            IssueCode::InvalidUnion => "invalid_union",
            IssueCode::TooSmall => "too_small",
            IssueCode::TooBig => "too_big",
            IssueCode::InvalidValue => "invalid_value",
            IssueCode::InvalidFormat => "invalid_format",
            IssueCode::InvalidKey => "invalid_key",
            IssueCode::Custom => "custom",
        }
    }
}

/// One step in an issue path. Maps to: zod/v4 `PropertyKey`.
#[derive(Clone, Debug, PartialEq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

/// Maps to: zod/v4 `ZodIssue`. Optional fields are omitted from the dump when
/// `None`, matching zod's presence/absence (e.g. only `too_small` carries
/// `minimum`, only `invalid_union` carries `errors`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Issue {
    pub code: IssueCode,
    /// `PropertyKey[]`; string for keys, number for indices.
    pub path: Vec<PathSegment>,
    /// zod/v4 issue `message`, verbatim.
    pub message: String,
    /// `invalid_type` / safeint-fract: the expected bucket (`string`, `int`,
    /// `record`, …).
    pub expected: Option<&'static str>,
    /// `invalid_type` safeint-fract: `format: "safeint"`; `invalid_format`: the
    /// check's name (`"regex"` / `"url"`).
    pub format: Option<&'static str>,
    /// `invalid_format` from `.regex` only: the pattern WITH its `/`
    /// delimiters, unlike the bare source in the JSON Schema.
    pub pattern: Option<String>,
    /// `too_small` / `too_big`: the bound's origin (`number` / `int`).
    pub origin: Option<&'static str>,
    /// `too_small` / `too_big`: the bound value.
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    /// `too_small` / `too_big`: whether the bound is inclusive.
    pub inclusive: Option<bool>,
    /// `too_small` / `too_big` from `.length(n)` only: zod marks the bound as
    /// an exact-length check (oracle: string_length_short / _long).
    pub exact: Option<bool>,
    /// `invalid_format` from `.startsWith(p)` only: the required prefix
    /// (oracle: string_starts_with_fail).
    pub prefix: Option<String>,
    /// `invalid_format` from `.endsWith(suffix)`.
    pub suffix: Option<String>,
    /// `too_big` int: the safe-integer note.
    pub note: Option<&'static str>,
    /// `custom` from `.superRefine` only: the `params` object the check
    /// attached (`PermissionRuleSchema`'s `{received}`).
    pub params: Option<Value>,
    /// `invalid_value`: the accepted literals / enum options.
    pub values: Option<Vec<Value>>,
    /// `unrecognized_keys`.
    pub keys: Vec<String>,
    /// `invalid_union`: one nested issue list per variant.
    pub errors: Vec<Vec<Issue>>,
    /// `invalid_key`: the nested issues explaining why the key failed
    /// (a single flat list, unlike `errors`).
    pub issues: Vec<Issue>,
}

impl Issue {
    /// The zod/v4 issue as a JSON object, in zod's property order — this is
    /// what `ZodError.message` dumps, so order is byte-significant.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        // zod's order varies by code; see the oracle samples in the tests.
        match self.code {
            IssueCode::TooSmall => {
                if let Some(o) = self.origin {
                    m.insert("origin".into(), json!(o));
                }
                m.insert("code".into(), json!(self.code.as_str()));
                if let Some(v) = self.minimum {
                    m.insert("minimum".into(), bound_to_json(v));
                }
                if let Some(i) = self.inclusive {
                    m.insert("inclusive".into(), json!(i));
                }
                // `.length(n)` marks the bound exact, after `inclusive`
                // (oracle: string_length_short).
                if let Some(e) = self.exact {
                    m.insert("exact".into(), json!(e));
                }
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            // zod reaches `too_big` by two different paths whose object
            // literals are built in different orders, and this dump is
            // byte-significant. The safe-integer ceiling (the one carrying
            // `note`) is code-first; a `.max()` size check is origin-first with
            // `inclusive`, exactly like `too_small`.
            IssueCode::TooBig if self.note.is_some() => {
                m.insert("code".into(), json!(self.code.as_str()));
                if let Some(v) = self.maximum {
                    m.insert("maximum".into(), bound_to_json(v));
                }
                if let Some(n) = self.note {
                    m.insert("note".into(), json!(n));
                }
                if let Some(o) = self.origin {
                    m.insert("origin".into(), json!(o));
                }
                if let Some(i) = self.inclusive {
                    m.insert("inclusive".into(), json!(i));
                }
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            IssueCode::TooBig => {
                if let Some(o) = self.origin {
                    m.insert("origin".into(), json!(o));
                }
                m.insert("code".into(), json!(self.code.as_str()));
                if let Some(v) = self.maximum {
                    m.insert("maximum".into(), bound_to_json(v));
                }
                if let Some(i) = self.inclusive {
                    m.insert("inclusive".into(), json!(i));
                }
                // `.length(n)` marks the bound exact, after `inclusive`
                // (oracle: string_length_long).
                if let Some(e) = self.exact {
                    m.insert("exact".into(), json!(e));
                }
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            // regex carries `origin` and `pattern`; `.startsWith` carries
            // `origin` and `prefix`; a format check such as `.url()` carries
            // none of those (oracle: string_regex / string_starts_with_fail /
            // string_url).
            IssueCode::InvalidFormat => {
                if let Some(o) = self.origin {
                    m.insert("origin".into(), json!(o));
                }
                m.insert("code".into(), json!(self.code.as_str()));
                if let Some(f) = self.format {
                    m.insert("format".into(), json!(f));
                }
                if let Some(p) = &self.pattern {
                    m.insert("pattern".into(), json!(p));
                }
                if let Some(p) = &self.prefix {
                    m.insert("prefix".into(), json!(p));
                }
                if let Some(suffix) = &self.suffix {
                    m.insert("suffix".into(), json!(suffix));
                }
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            IssueCode::InvalidType => {
                if let Some(e) = self.expected {
                    m.insert("expected".into(), json!(e));
                }
                if let Some(f) = self.format {
                    m.insert("format".into(), json!(f));
                }
                m.insert("code".into(), json!(self.code.as_str()));
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            IssueCode::InvalidValue => {
                m.insert("code".into(), json!(self.code.as_str()));
                if let Some(v) = &self.values {
                    m.insert("values".into(), json!(v));
                }
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            IssueCode::UnrecognizedKeys => {
                m.insert("code".into(), json!(self.code.as_str()));
                m.insert("keys".into(), json!(self.keys));
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            IssueCode::InvalidUnion => {
                m.insert("code".into(), json!(self.code.as_str()));
                m.insert(
                    "errors".into(),
                    Value::Array(
                        self.errors
                            .iter()
                            .map(|variant| {
                                Value::Array(variant.iter().map(Issue::to_json).collect())
                            })
                            .collect(),
                    ),
                );
                if let Some(n) = self.note {
                    m.insert("note".into(), json!(n));
                }
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            // `invalid_key` nests the key's own failure under `issues`
            // (oracle: partial_record_bad_key — origin/code/issues/path/message).
            IssueCode::InvalidKey => {
                if let Some(o) = self.origin {
                    m.insert("origin".into(), json!(o));
                }
                m.insert("code".into(), json!(self.code.as_str()));
                m.insert(
                    "issues".into(),
                    Value::Array(self.issues.iter().map(Issue::to_json).collect()),
                );
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
            // A user-supplied `addIssue` object dumps in ITS key order plus
            // the appended path — `{code, message, params}` gives
            // code/message/params/path (oracle: super_refine_params), while a
            // zod-built refine issue stays code/path/message.
            IssueCode::Custom if self.params.is_some() => {
                m.insert("code".into(), json!(self.code.as_str()));
                m.insert("message".into(), json!(self.message));
                m.insert("params".into(), self.params.clone().unwrap());
                m.insert("path".into(), path_to_json(&self.path));
            }
            IssueCode::Custom => {
                m.insert("code".into(), json!(self.code.as_str()));
                m.insert("path".into(), path_to_json(&self.path));
                m.insert("message".into(), json!(self.message));
            }
        }
        Value::Object(m)
    }
}

/// A numeric bound as zod writes it: JS has no int/float split, so a whole
/// number serialises as `0`, not `0.0`.
pub(super) fn bound_to_json(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_991.0 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

fn path_to_json(path: &[PathSegment]) -> Value {
    Value::Array(
        path.iter()
            .map(|seg| match seg {
                PathSegment::Key(k) => json!(k),
                PathSegment::Index(i) => json!(i),
            })
            .collect(),
    )
}

/// Maps to: zod/v4 `ZodError`. `message` is `JSON.stringify(issues, null, 2)`
/// of the whole tree — the model sees it via the formatter's fallback and via
/// `toolUseResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct ZodError {
    pub issues: Vec<Issue>,
}

impl ZodError {
    pub(crate) fn new(issues: Vec<Issue>) -> Self {
        ZodError { issues }
    }

    /// zod/v4 `error.message` — the pretty-printed issues dump.
    pub fn message(&self) -> String {
        let arr = Value::Array(self.issues.iter().map(Issue::to_json).collect());
        serde_json::to_string_pretty(&arr).unwrap_or_default()
    }
}
