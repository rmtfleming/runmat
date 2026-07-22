# Tasks — 005-struct-cell-access

Order is normative (test-first). Every task cites requirements.

- [ ] **T1** Inspect `Value`/`CellArray` storage conventions (row-major cell
  data vs column-major tensor data) and the `fieldnames` delegation path
  (`call_builtin_async`, per `bounds`). [FR-005-01, FR-005-04] (no code
  changes)
- [ ] **T2** Author tests for the `fields` observed cases (`two-fields` →
  2×1 cell, `empty-struct` → 0×1 cell) plus `unresolved_choice` tests for
  non-struct and struct-array inputs. [FR-005-01..03; SC-005-1]
- [ ] **T3** Author tests for the `num2cell` observed cases (`vector`,
  `matrix`, `dim1`, `scalar`), the `summary_derived` dim-1 column-contents
  test, and `unresolved_choice` tests (empty, dim vector, char, invalid
  dims). [FR-005-04..06; SC-005-1]
- [ ] **T4** Implement `fields.rs` (GPU/fusion specs, descriptor, macro,
  delegation body) + `mod` entry in `structs/core/mod.rs`. [FR-005-01..03]
- [ ] **T5** Implement `num2cell.rs` (GPU/fusion specs, descriptor, macro,
  kind normalization, grouping, row-major cell assembly) + `mod` entry in
  `cells/core/mod.rs` + `num2cell_type` resolver. [FR-005-04..06]
- [ ] **T6** Run verification commands; record output in `validation.md`.
  [SC-005-1..3]
- [ ] **T7** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-005-3; Principle VI]
