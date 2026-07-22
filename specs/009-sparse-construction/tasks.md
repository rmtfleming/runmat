# Tasks — 009-sparse-construction

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `SparseTensor` (CSC constructors, validation,
  `to_dense`) and the existing `array/creation/sparse.rs` conventions;
  confirm value mapping for sparse/dense results. [FR-009-01, FR-009-04,
  FR-009-07] (no code changes)
- [x] **T2** Author tests for the observed `full` cases (sparse matrix,
  sparse vector, dense passthrough, empty sparse) with exact CSC/dense
  expectations. [FR-009-01, FR-009-02; SC-009-1]
- [x] **T3** Author tests for the observed `speye` and `spdiags` cases
  (square/rectangular/size-vector/zero identity; extract/construct), plus
  `summary_derived` tests (spdiags second output, sparse-input extraction)
  and `unresolved_choice` tests (padding rule, m < n round trip, invalid
  inputs/forms). [FR-009-04..05, FR-009-07..10; SC-009-1]
- [x] **T4** Implement `math/sparse/full.rs` + registration (descriptor,
  GPU/fusion specs, macro, mod entry). [FR-009-01..03]
- [x] **T5** Implement `math/sparse/speye.rs` + registration (direct CSC
  identity construction). [FR-009-04..06]
- [x] **T6** Implement `math/sparse/spdiags.rs` + registration (extraction,
  construction, multi-output via `crate::output_count`). [FR-009-07..10]
- [x] **T7** Run verification commands; record output in `validation.md`.
  [SC-009-1..3]
- [x] **T8** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-009-3; Principle VI]
