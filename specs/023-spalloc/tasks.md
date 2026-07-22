# Tasks — 023-spalloc

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Confirm the empty-CSC construction path: read
  `SparseTensor::zeros` in `crates/runmat-builtins/src/lib.rs` and the `speye`
  exemplar; confirm the all-zero `m`-by-`n` mapping (nnz = 0).
  [FR-023-03] (no code changes)
- [x] **T2** Author the observed-tier test `observed_basic_all_zero_sparse`
  (spalloc(3,3,4) ⇒ zeros(3,3), size [3,3], nnz 0) in the new file's test
  module. [FR-023-01, FR-023-02, FR-023-03; SC-023-1]
- [x] **T3** Author unresolved-choice tests: nz capacity ignored, rectangular /
  empty sizes, scalar-like kinds, invalid dims, invalid nz, wrong arg count,
  type resolver. [FR-023-04, FR-023-05]
- [x] **T4** Implement `spalloc.rs` (descriptor, GPU/fusion specs,
  `#[runtime_builtin]`, `spalloc_builtin`, `spalloc_type`, error builders)
  building `SparseTensor::zeros(m, n)`. [FR-023-01..05]
- [x] **T5** Register: add `pub(crate) mod spalloc;` to
  `math/sparse/mod.rs`. [FR-023-01; SC-023-2]
- [x] **T6** Run verification commands; record output in `validation.md`.
  [SC-023-1, SC-023-2]
- [x] **T7** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-023-3; Principle VI]
