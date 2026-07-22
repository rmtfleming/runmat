# Tasks — 017-nonzeros

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `SparseTensor` (CSC invariants: `col_ptrs`,
  `row_indices` sorted per column, `values`) and `Tensor` column-major
  storage; confirm the value array is already column-major and identify the
  house result-normalisation helpers (`tensor_into_value`,
  `complex_tensor_into_value`, `column_vector_type`). [FR-017-02,
  FR-017-03] (no code changes)
- [x] **T2** Author tests for the four observed cases (`row`, `matrix`,
  `col-major` order-discriminating, `sparse` 1×1) with exact value/shape
  expectations, plus the `summary_derived` multi-column sparse case.
  [FR-017-01..03; SC-017-1]
- [x] **T3** Author `unresolved_choice_*` tests for the documented choices:
  all-zero → 0×1 empty (dense and sparse), scalar inputs, NaN kept /
  negative zero dropped, logical array → double, complex tensor/scalar,
  N-D column-major, non-numeric input error. [FR-017-04, FR-017-05]
- [x] **T4** Implement `math/sparse/nonzeros.rs` (descriptor, GPU/fusion
  specs, `runtime_builtin` macro, type resolver, error builders) and
  register it in `math/sparse/mod.rs`. [FR-017-01..05]
- [x] **T5** Run verification commands (`cargo check`, targeted tests,
  `cargo fmt --all -- --check`); record output in `validation.md`.
  [SC-017-1..2]
- [x] **T6** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-017-3; Principle VI]
