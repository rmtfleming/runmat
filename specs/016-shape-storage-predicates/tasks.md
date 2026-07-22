# Tasks — 016-shape-storage-predicates

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect `Value` variants and `common::shape::value_dimensions`
  for shape/storage representation (scalar → `[1,1]`, sparse kind, cell
  elements, GPU handle metadata); confirm predicate mappings; confirm no
  table kind exists (Blocker B-010-1). [FR-016-01..05] (no code changes)
- [x] **T2** Author `observed_*` tests for the exact observed cases of all
  five builtins in the new files' test modules (red step; compile gated by
  T4–T8). [FR-016-01..05; SC-016-1]
- [x] **T3** Author `unresolved_choice_*` tests for documented choices:
  empty/N-D shapes, GPU handles, string elements, non-sparse kinds,
  always-false `istable` kinds. [FR-016-06]
- [x] **T4** Implement `isrow.rs` + registration (GPU/fusion specs,
  descriptor, macro, mod entry). [FR-016-01]
- [x] **T5** Implement `iscolumn.rs` + registration. [FR-016-02]
- [x] **T6** Implement `issparse.rs` + registration. [FR-016-03]
- [x] **T7** Implement `iscellstr.rs` + registration. [FR-016-04]
- [x] **T8** Implement `istable.rs` (always-false, gate-ratified; document
  the unreachable observed true-case citing istable.logical-table-test) +
  registration. [FR-016-05]
- [x] **T9** Run verification commands; record output in `validation.md`.
  [SC-016-1..3]
- [x] **T10** Traceability table in `validation.md` (claim → FR → test →
  result). [SC-016-3; Principle VI]
