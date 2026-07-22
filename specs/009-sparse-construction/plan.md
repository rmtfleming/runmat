# Implementation Plan — 009-sparse-construction

## Constitution Check

- I Specification-first: requirements cite approved claims — PASS
- II Clean-room: sources = imported export + RunMat code only; no MATLAB — PASS
- III Independence: CSC construction/expansion derived from RunMat's own
  `SparseTensor` model; diagonal rule derived from the observed cases and
  documented as an independent choice — PASS
- IV Provenance: `provenance.md` complete; interim pin waiver recorded — PASS
- V Test-first: observed-case tests authored before builtin bodies were
  finalised (tasks T2–T3 before T4–T6) — PASS
- VII Minimal scope: three tightly coupled sparse-construction builtins, no
  refactoring of the existing `sparse` builtin — PASS
- IX Compatibility: follows `math/reduction/` module conventions,
  `#[runtime_builtin]` registration, descriptor + GPU/fusion spec pattern — PASS

## Affected crates and files

| Item | Path |
|---|---|
| Module | `crates/runmat-runtime/src/builtins/math/sparse/mod.rs` (new; shared size/scalar helpers) |
| `full` | `crates/runmat-runtime/src/builtins/math/sparse/full.rs` (new) |
| `speye` | `crates/runmat-runtime/src/builtins/math/sparse/speye.rs` (new) |
| `spdiags` | `crates/runmat-runtime/src/builtins/math/sparse/spdiags.rs` (new) |
| Registration | add `pub mod sparse;` in `crates/runmat-runtime/src/builtins/math/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplars: `logical/tests/iscell.rs` and `math/reduction/bounds.rs` (file
structure, GPU/fusion specs, descriptors, error builders, inline tests);
`io/repl_fs/fileparts.rs` (multi-output via `crate::output_count`);
`array/creation/sparse.rs` (CSC assembly and size/subscript parsing
conventions — note this existing builtin already produces
`Value::SparseTensor`; feature 009 constructs `SparseTensor` directly and
does not call it).

## Design decisions

- Substrate: `runmat_builtins::SparseTensor` (CSC: `rows`, `cols`,
  `col_ptrs`, sorted unique `row_indices`, `values`) with validated
  `SparseTensor::new` and `to_dense()` (column-major expansion).
- `full`: `Value::SparseTensor` → `to_dense()` → `Value::Tensor`; every
  other value kind passes through unchanged (documented choice on
  unobserved kinds; GPU tensors are not gathered — they are already full).
  Densification failure (dimension overflow) → `RunMat:full:Internal`.
- `speye`: parses `(n)`, `(m, n)`, `([m n])`; builds the identity CSC
  directly (unit entry at (i, i), i < min(m, n)); result is always
  `Value::SparseTensor`. Invalid dimensions → `RunMat:speye:InvalidArgument`.
- `spdiags`: one argument = extraction, four arguments = construction; any
  other count → `RunMat:spdiags:InvalidArgument` (spdiags.q-forms).
  Diagonal rule (documented, reproduces observations exactly): d = j − i;
  extraction returns min(m, n)×p dense B, diagonals ascending; row in B is
  j for m ≥ n, i for m < n; construction is the inverse mapping with
  out-of-range positions ignored and zeros unstored. Second output `d`
  (p×1) served through `crate::output_count` (summary-derived). Sparse
  inputs are enumerated per stored entry (no densification of A); GPU
  inputs are gathered first.
- Output modes: `Fixed` for `full`/`speye`;
  `ByRequestedOutputCount` for `spdiags`.
- Type resolvers: `full` mirrors its input type; `speye`/`spdiags` return
  `Type::tensor()` (sparse values are typed `Type::Tensor` in RunMat).

## Test plan (inline `#[cfg(test)]`, per house convention)

Normative (cite FR/claims in test comments):
- `full`: sparse 2×2 → dense `[1 0;0 2]`; sparse 1×3 → `[0 3 0]`; dense
  passthrough `[1 2;3 4]`; empty sparse → 0×0 dense.
- `speye`: `speye(3)`, `speye(2, 4)`, `speye([3 3])`, `speye(0)` compared
  against exact CSC expectations.
- `spdiags`: extract observed 3×3 → dense `[1 0;3 2;5 4]`; construct
  observed `([1;2;3], 0, 3, 3)` → exact CSC expectation.
Summary-derived (tagged `summary_derived`): spdiags second output `d`;
spdiags extraction from a sparse input.
Unresolved-tracking (tagged `unresolved_choice`): full non-sparse kind
passthrough; speye invalid sizes; spdiags subdiagonal padding, m < n round
trip, unsupported call forms.
Plus error-path tests (internal overflow, size mismatch, argument counts).

## Risks & rollback

- Risk: diagonal layout rule wrong — mitigated by asserting the observed
  extract case element-for-element and a construct/extract round trip.
- Risk: WASM/registry regeneration — handled by macro `builtin_path`;
  verified with `cargo check -p runmat-runtime`.
- Rollback: delete `math/sparse/` and the single `pub mod sparse;` line in
  `math/mod.rs` (no other existing file is modified).

## Verification commands

`cargo fmt --all -- --check` · `cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse`
