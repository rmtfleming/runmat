# Implementation Plan — 038-validateattributes (spec 0.3.0)

## Constitution Check

- I Specification-first: FR-038-01..08 cite 0.3.0 claims (all 43 ids
  verified); FR-038-09..16 are labelled documented choices; unresolved
  points are listed with dispositions. PASS.
- II Clean-room: inputs are the imported export plus RunMat code; MATLAB not
  invoked; observed `MATLAB:` identifiers are mapped, never emitted. PASS.
- III Independence: checks are direct definitions over RunMat's `class()`,
  `size()` and element values. PASS.
- IV Provenance: pinned to `8614c5b`, freshness check recorded. PASS.
- V Test-first: the 0.3.0 tests are written and run (failing with
  `UndefinedFunction`) before the builtin is implemented. PASS.
- VII Minimal scope: one builtin; one new file plus one `pub mod` line. PASS.
- IX Compatibility: `#[runtime_builtin]` with a descriptor, `accel = "cpu"`,
  `sink = true`, `suppress_auto_output = true` (the `mustBe*` convention);
  warnings through `crate::warnings::emit`; no integer-capability metadata
  (same reason as feature 034). PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/common/validateattributes.rs` |
| Registration | `pub mod validateattributes;` in `crates/runmat-runtime/src/builtins/common/mod.rs` |

`common/mod.rs` is `@generated`; no hand-written parent fits an
argument-validation builtin. The one-line edit is flagged for the
maintainer (a regeneration must include the new file).

## Design

1. `current_output_count()` > 0 → `maxlhs` (FR-038-06).
2. Arity 3..=6 else `InvalidArgument`.
3. Parse `classes` → names (FR-038-14); empty list → `invalidType` at the
   class check (FR-038-02).
4. Parse `attributes` → `Vec<Item>`: `Known(Attr)` (31 names, value-taking
   ones consume the next entry), `Unknown(name)` (warn), with
   `UnsupportedOption` for case variants, `'diag'`, `'scalartext'`,
   non-text names and malformed values; missing value → `InvalidArgument`.
5. Parse trailing context (`funcName`, `varName`, `argIndex`) → message
   prefix only.
6. Class check → `invalidType`.
7. Gather a GPU value once; walk the items in order: shape checks from
   `value_dimensions`; value checks over an `Elems` view
   (`Real(Vec<f64>)`, `Int(Vec<i128>)`, `Complex(Vec<(f64,f64)>)`, or
   none for non-numeric classes → `UnsupportedOption`); `Unknown` →
   warning `attributeNotFound`.
8. Return `Value::Num(0.0)` (validator convention; sink, output
   suppressed).

Errors: one `BuiltinErrorDescriptor` per RunMat identifier in the spec's
mapping table, built with a local macro.

## Test plan (inline `#[cfg(test)] mod validateattributes_tests`)

- `observed_*`: one table-driven test per group of the 58 cases, asserting
  pass + no warning, the mapped RunMat identifier, or the warning.
- `documented_choice_*`: numeric category over integers, logical values,
  string-array classes, NaN/Inf handling, complex finite/nonzero, N-D
  `size` padding, `nondecreasing` identifier, message context text.
- `unresolved_choice_*`: q-more-attributes, case variants, `'diag'`,
  `'scalartext'`, float/integer categories, monotonic on matrices,
  complex ordering, malformed values and trailing args.
- `rejects_*`: arity, malformed class/attribute lists, missing values.

## Risks & rollback

- Regeneration of `common/mod.rs` could drop the line (flagged).
- Rollback: delete the file and its `mod` line.

## Verification commands

`cargo test -p runmat-runtime --lib -- validateattributes` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short` (filtered
to the file) · `rustfmt --edition 2021 --check <file>`
