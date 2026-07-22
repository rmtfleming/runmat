# Implementation Plan — 005-struct-cell-access

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: `fields` derived from RunMat's own `fieldnames`
  delegation; `num2cell` derived from RunMat's column-major tensor /
  row-major cell value model — PASS
- IV Provenance: `provenance.md` complete; interim pin waiver recorded — PASS
- V Test-first: observed-case tests authored with the builtin bodies
  (tasks T2–T3 before/with T4–T5) — PASS
- VII Minimal scope: two container-access builtins, no refactoring — PASS
- IX Compatibility: follows `structs/core/` and `cells/core/` conventions — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `fields` | `crates/runmat-runtime/src/builtins/structs/core/fields.rs` (new) |
| `num2cell` | `crates/runmat-runtime/src/builtins/cells/core/num2cell.rs` (new) |
| Registration | add `mod` entries in `structs/core/mod.rs` and `cells/core/mod.rs` |
| Type resolver | add `num2cell_type` in `cells/type_resolvers.rs`; reuse `structs/type_resolvers::fieldnames_type` for `fields` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `structs/core/fieldnames.rs` (cell-of-names construction — reused
via delegation, not modified), `math/reduction/bounds.rs` (builtin-to-builtin
delegation through `call_builtin_async`), `cells/core/mat2cell.rs` (input
kind handling, column-major block extraction, GPU gather, cell assembly),
`logical/tests/iscell.rs` (GPU/fusion spec + descriptor house style).

## Design decisions

- `fields`: delegate to the registered `fieldnames` builtin via
  `crate::call_builtin_async("fieldnames", &[value])` — the approved summary
  describes `fields` as a legacy equivalent of `fieldnames`, and delegation
  guarantees the observed N×1 cell shape (0×1 for no fields) without
  duplicating the collection logic. Errors surface from the delegate
  (documented choice; error inputs unobserved).
- `num2cell`: normalize the input into one of five kinds (real tensor,
  complex tensor, logical, string array, char array; scalars promoted to
  1×1). Compute an outer cell shape (grouped dims → 1) and a per-cell
  content shape (grouped dims → full extent, else 1). Iterate the outer
  shape row-major (cell-array storage order) and extract each block
  column-major (tensor storage order). Single-element blocks collapse to
  scalar values (`Value::Num`/`Bool`/`Complex`/`String`), matching
  `mat2cell`; char slices stay `CharArray` (row-major addressing).
- Dimension argument: one optional argument, scalar or vector of positive
  integers; sorted/deduped; dims beyond `ndims` act as trailing singletons.
  More than one extra argument is rejected.
- GPU handles: `gather_if_needed_async` before conversion (residency
  `GatherImmediately`), mirroring `mat2cell`.
- Output mode `Fixed` for both; type resolvers `fieldnames_type` (cell of
  string) and new `num2cell_type` (cell of element-type union, sharing
  `mat2cell`'s element mapping).

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test comments):
- fields: `struct('a',1,'b',2)` → 2×1 cell; `struct()` → 0×1 cell.
- num2cell: `[10 20 30]` → 1×3 cell of 1×1 elements; `[1 2; 3 4]` → 2×2
  cell; `[1 2; 3 4]` with dim 1 → 1×2 cell; `7` → 1×1 cell.
Summary-derived (tagged `summary_derived` in name): dim-1 cells each hold a
2×1 column (approved claim statement; contents unobserved).
Unresolved-tracking (tagged `unresolved_choice`): fields on non-struct
errors; fields on struct array matches fieldnames; num2cell on empty → 0×0
cell; dim vector `[1 2]` → 1×1 cell; char row → char cells; invalid dims
error.
Plumbing (no conformance claim): descriptor signature labels; GPU gather
fallback.

## Risks & rollback

- Risk: cell-array storage order (row-major) vs tensor order (column-major)
  mismatch would scramble `num2cell` output — covered by the matrix test
  asserting per-position contents via `CellArray::get`.
- Risk: delegation requires the registry to be populated in unit tests —
  already proven by `bounds` (feature 004) under `RUST_TEST_THREADS=1`.
- Rollback: delete the two new files + `mod` entries + `num2cell_type`
  (no existing behaviour is modified).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::structs::core::fields builtins::cells::core::num2cell`
