# Implementation Plan — 037-findstr (spec 0.3.0)

## Constitution Check

- I Specification-first: FR-037-01..07 cite the 14 approved 0.3.0 claims.
  FR-037-08..12 are tagged documented choice or unresolved. PASS.
- II Clean-room: inputs are the imported export plus RunMat code only;
  MATLAB not invoked. PASS.
- III Independence: a naive left-to-right window scan (textbook). PASS.
- IV Provenance: `provenance.md` pinned to `8614c5b`, freshness-checked,
  with all 14 claim ids verified. PASS.
- V Test-first: the 0.3.0 tests were written and run against a test-only
  module (the 0.2.0-era implementation was removed) and failed with
  `UndefinedFunction` before the implementation was added. PASS.
- VII Minimal scope: one new file plus one `pub mod findstr;` line in
  `strings/search/mod.rs`. **That parent is generated ("Do not edit").** The
  line follows the `strmatch` precedent and must be listed in the maintainer
  README of generated-parent edits. PASS (flagged).
- IX Compatibility: `builtins/strings/search/`, `#[runtime_builtin]` with a
  descriptor, `accel = "cpu"`. No integer-capability metadata. PASS.

## Affected files

| Item | Path |
|---|---|
| Builtin (new) | `crates/runmat-runtime/src/builtins/strings/search/findstr.rs` |
| Registration | `pub mod findstr;` in `crates/runmat-runtime/src/builtins/strings/search/mod.rs` (generated parent; flagged) |

## Design

1. `findstr_builtin(args)` takes variadic arguments, so the builtin owns the
   arity errors: fewer than 2 → `NotEnoughInputs`, more than 2 →
   `TooManyInputs`.
2. Gather each argument and convert it to a `Sequence`.
   - Text:
     - A char with at most one row (including `0×0`) is text.
     - Multi-row char → `MultiRowInput`.
     - A string scalar or one-element string array is text.
   - Numbers:
     - A real double scalar or row (an empty double counts as empty) gives
       numbers.
     - Multi-row double → `MultiRowInput`.
   - Integer, logical, single or complex → `UnsupportedOption`.
   - Anything else → `InputsMustBeCharArrays`.
3. Search the shorter sequence within the longer (s1 on equal lengths). Two
   different kinds (text and numbers) → `UnsupportedOption`.
4. Collect every 1-based window start, overlaps included. An empty needle
   gives no matches.
5. Output: `[1 n]` double, or `0×0` when there are no matches.

## Test plan (inline `#[cfg(test)] mod findstr_tests`, via `crate::call_builtin_async`)

- One `observed_*` test per observed case (17).
- `documented_choice_text_forms`, `documented_choice_numeric_forms`,
  `documented_choice_other_inputs_and_arity`.
- `unresolved_unobserved_classes_are_unsupported` (U-037-1).

## Risks & rollback

- Regenerating the module composition would drop `pub mod findstr;` unless
  the generator learns about it (same situation as `strmatch`).
- Rollback: delete `findstr.rs` and its `mod` line.

## Verification commands

`cargo test -p runmat-runtime --lib -- findstr_tests` ·
`cargo test -p runmat-runtime --lib -- strings::search` (no regressions) ·
`rustfmt --edition 2021 --check <file>` ·
`cargo clippy -p runmat-runtime --lib --tests --message-format short | grep findstr.rs`
