# Tasks — 034-table-variable-editing

Order is normative (test-first). Every task cites requirements.

- [x] **T1** Inspect the upstream table subsystem (variables struct,
  Properties, default names, `into_table_object`, error helpers); confirm
  placement in the hand-written `table/builtins/` tree. [FR-034-01..03]
  (no code changes)
- [x] **T2** Author the three observed normative tests (addvars `append`,
  removevars `by-name`, movevars `before`/`order`). [FR-034-01..03; SC-034-1]
- [x] **T3** Author unresolved-disposition tests for every US4 case.
  [FR-034-04..06]
- [x] **T4** Author documented-choice and validation tests: default name,
  height mismatch, unknown names, non-table, timetable, bad keyword,
  string-scalar names, metadata preservation, self-move. Run the test
  filter and record that it fails (builtins absent) before T5.
  [FR-034-07..10]
- [x] **T5** Implement `variable_editing.rs`: descriptors, error sets,
  shared `rebuild` core, and the three builtins (integer-capability
  metadata deliberately omitted; see plan.md). [FR-034-01..10]
- [x] **T6** Register: `mod variable_editing;` in `table/builtins.rs`
  (the inventory macro registers the builtins; no re-export needed). [SC-034-2]
- [x] **T7** Run the verification commands; record the output in
  `validation.md`. [SC-034-1..3]
- [x] **T8** Traceability table (claim → FR → test → result) in
  `validation.md`. [SC-034-3; Principle VI]
