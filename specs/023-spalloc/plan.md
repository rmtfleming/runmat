# Implementation Plan — 023-spalloc

## Constitution Check

- I Specification-first: every FR cites an approved claim id — PASS
- II Clean-room: sources = imported `spalloc` 0.2.0 export + RunMat code only;
  no MATLAB invoked — PASS
- III Independence: result is RunMat's own empty-CSC constructor
  (`SparseTensor::zeros`); no MATLAB internals imitated — PASS
- IV Provenance: `provenance.md` complete; commit pin `4445294` recorded — PASS
- V Test-first: acceptance tests authored alongside the builtin; observed case
  is the red/green anchor — PASS
- VII Minimal scope: one builtin, no refactoring; reuses existing sparse
  helpers — PASS
- IX Compatibility: follows `math/sparse/speye.rs` conventions (descriptor,
  GPU/fusion specs, `#[runtime_builtin]`, `Value::SparseTensor`) — PASS
- X Licensing: no external algorithm source needed; trivial empty constructor — PASS

## Affected crates and files

| Item | Path |
|---|---|
| `spalloc` implementation | `crates/runmat-runtime/src/builtins/math/sparse/spalloc.rs` (new) |
| Registration | add `pub(crate) mod spalloc;` to `crates/runmat-runtime/src/builtins/math/sparse/mod.rs` |
| Docs metadata | descriptor + `#[runtime_builtin]` doc fields (feeds `docs/builtins` tooling) |

Exemplar: `math/sparse/speye.rs` (SparseTensor construction, descriptor, GPU
spec, fusion spec, error builders, inline test module). SparseTensor
constructor: `crates/runmat-builtins/src/lib.rs` — `SparseTensor::zeros(m, n)`
yields the empty CSC form (col_ptrs all zero, empty row_indices/values).

## Design decisions

- Signature `spalloc_builtin(args: Vec<Value>)`: exactly three arguments; this
  gives full control over arity errors (missing `nz` / too many args) with a
  `spalloc`-identified error, matching the module's explicit-arity style.
- Dimensions `m`, `n` and hint `nz` parsed with the sparse module's shared
  `scalar_f64` + `size_from_f64` (nonnegative-integer, scalar-like), the same
  helpers `speye` uses.
- `nz` validated but not used for content — RunMat CSC has no reserved capacity,
  so `nnz` is always 0 (FR-023-04). Documented choice `spalloc.q-nz-capacity`.
- Result built via `SparseTensor::zeros(m, n)` → `Value::SparseTensor`.
- `type_resolver` `spalloc_type` returns `Type::tensor()` (sparse values are
  typed `Type::Tensor` in RunMat), mirroring `speye`.
- Output mode `Fixed`; `accel = "array_construct"`; GPU/fusion specs mark it a
  host-resident non-fusible sparse constructor (mirror `speye`).
- Single error descriptor `RM.SPALLOC.INVALID_ARGUMENT`
  (`RunMat:spalloc:InvalidArgument`).

## Test plan (inline `#[cfg(test)]`, per house convention)

Observed tier (normative, cite FR/claims in comments):
- `observed_basic_all_zero_sparse`: `spalloc(3,3,4)` ⇒ `SparseTensor::zeros(3,3)`,
  size `[3,3]`, `nnz == 0`.

Unresolved-choice tier (tagged `unresolved_choice_*`, non-normative):
- `nz` capacity does not change the result (0, 4, 100 all identical).
- Rectangular and empty sizes follow the all-zero rule.
- Scalar-like argument kinds (Int, Bool, 1-element tensor) accepted.
- Invalid dimensions (negative, non-integer, non-scalar) error.
- Invalid `nz` (negative, non-integer) error.
- Wrong argument count (missing `nz`, too many) error.
- `spalloc_type` resolves to `Type::tensor()`.

## Risks & rollback

- Risk: concurrent edits to `math/sparse/mod.rs` by sibling batch features —
  handled by a single additive `mod` line; if a compile error is not in
  `spalloc.rs`, wait and re-run rather than edit sibling files.
- Rollback: delete `spalloc.rs` and remove the one `mod.rs` line (no existing
  file is otherwise modified).

## Verification commands

`cargo check -p runmat-runtime` ·
`RUST_TEST_THREADS=1 cargo test -p runmat-runtime --lib -- builtins::math::sparse::spalloc` ·
`cargo fmt` then `cargo fmt --all -- --check`
