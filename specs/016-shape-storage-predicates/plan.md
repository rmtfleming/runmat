# Implementation Plan — 016-shape-storage-predicates

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: predicates derived from RunMat `Value` kinds and RunMat's
  own dimension metadata — PASS
- IV Provenance: `provenance.md` complete; commit pin
  `e46587ae1b78237e0f43cd55f678de01a28495a4` — PASS
- V Test-first: tests authored with the builtin files before verification
  (tasks T2–T3 before T4–T8) — PASS
- VI Traceability: claim → FR → test table in `validation.md` — PASS
- VII Minimal scope: five tightly coupled predicates, no refactoring — PASS
- VIII Human gates: Gates 1–3 approved (Tier B batch 1 human-loop record,
  2026-07-22), including the always-false `istable` decision — PASS
- IX Compatibility: follows `logical/tests/` conventions (exemplars
  `iscell.rs`, `isstruct.rs`, `isvector.rs`) — PASS
- X Licensing: no external implementation sources used — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `isrow` | `crates/runmat-runtime/src/builtins/logical/tests/isrow.rs` (new) |
| `iscolumn` | `crates/runmat-runtime/src/builtins/logical/tests/iscolumn.rs` (new) |
| `issparse` | `crates/runmat-runtime/src/builtins/logical/tests/issparse.rs` (new) |
| `iscellstr` | `crates/runmat-runtime/src/builtins/logical/tests/iscellstr.rs` (new) |
| `istable` | `crates/runmat-runtime/src/builtins/logical/tests/istable.rs` (new) |
| Registration | add five `pub mod` entries in `logical/tests/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `logical/tests/iscell.rs` / `isstruct.rs` (file structure:
GPU_SPEC, FUSION_SPEC, descriptor consts, `#[runtime_builtin]` macro,
`bool_scalar_type`, helper fn, inline test module) and
`array/introspection/isvector.rs` (shape reasoning via
`builtins::common::shape::value_dimensions`, GPU handle metadata tests).

## Design decisions

- `isrow`: `value_dimensions(&value).await?` then
  `dims.len() == 2 && dims[0] == 1`. Scalars normalise to `[1, 1]` (both
  row and column — observed). GPU handles answer from `handle.shape`
  metadata; `value_dimensions` gathers only when a provider omits shape.
- `iscolumn`: same metadata source, `dims.len() == 2 && dims[1] == 1`.
- `issparse`: `matches!(value, Value::SparseTensor(_))` — RunMat's sole
  sparse representation. GPU handles are dense storage → `false`, no
  provider call (metadata-only, mirrors `iscell`).
- `iscellstr`: `Value::Cell` whose every element is char-like
  (`Value::CharArray` normative; `Value::String` accepted as a documented
  choice — RunMat's internal string-scalar text representation, unresolved
  `iscellstr.q-strings`). Empty cell → `true` (all-of-empty). All other
  kinds → `false`.
- `istable`: always `false` for every value kind (gate-ratified: RunMat has
  no table type, Blocker B-010-1). The observed true-case
  (istable.logical-table-test, case `table`) is unreachable — documented in
  code comment and `spec.md`; the feature returns to planning when a table
  type lands.
- Output mode `Fixed`, type_resolver `bool_scalar_type`, `accel =
  "metadata"`, empty error descriptor for all five.

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative `observed_*` tests (exact observed inputs; cite FR/claims):
- isrow: `[1 2 3]`→true, `[1;2;3]`→false, `5`→true, `[1 2;3 4]`→false.
- iscolumn: `[1;2;3]`→true, `[1 2 3]`→false, `5`→true, `[1 2;3 4]`→false.
- issparse: sparse `[1 0;0 2]`→true, full `[1 2;3 4]`→false,
  `sparse([])` (0×0)→true.
- iscellstr: `{'a','bc'}`→true, `{'a',1}`→false, `'abc'`→false, `{}`→true.
- istable: `[1 2;3 4]`→false, `{1,2}`→false.

Unresolved-tracking `unresolved_choice_*` tests (documented choices):
- isrow/iscolumn: empty shapes (`1×0`/`0×1`/`0×0`), N-D arrays → false,
  GPU handle answers from shape metadata.
- issparse: non-array kinds → false; GPU tensor → false without gather.
- iscellstr: `Value::String` elements → true; `StringArray` input → false;
  char-matrix element → true (kind-level); non-char kinds inside cell →
  false.
- istable: every constructible kind (struct, char, string, sparse, bool,
  GPU handle) → false.

## Risks & rollback

- Risk: five sibling features build concurrently in this tree — verify
  with narrowly scoped test filters; re-run rather than editing files
  owned by other features.
- Risk: `value_dimensions` semantics for N-D/empty shapes — covered by
  `unresolved_choice_*` tests pinning the documented choices.
- Rollback: delete the five new files + `mod.rs` lines (no other existing
  file is modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib --
builtins::logical::tests::isrow builtins::logical::tests::iscolumn
builtins::logical::tests::issparse builtins::logical::tests::iscellstr
builtins::logical::tests::istable` ·
`cargo fmt -p runmat-runtime` then `cargo fmt --all -- --check`
