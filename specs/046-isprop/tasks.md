# Tasks — 046-isprop

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  HEAD = origin/main = `8f72edb`, with no diff since `c65e238` and no
  working-tree change under `specifications/isprop`. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (pinned to `c65e238`;
  7 claim ids verified). [FR-046-01..12]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** Write the tests: 11 `observed_*` tests plus the
  documented-choice and unresolved tests. [FR-046-01..12]
- [x] **T4** Register the test-only module (`pub mod isprop;` in the
  generated `introspection/mod.rs`, flagged). Run the filter and record the
  failing run. [Principle V]
- [x] **T5** Implement `isprop.rs` (graphics, class objects, MException,
  tables, non-objects). [FR-046-01..12]
- [x] **T6** Run the verification commands: feature tests, the
  `introspection` regressions, clippy and rustfmt. Record the results in
  `validation.md`. [SC-046-1..3]
- [x] **T7** Write the traceability table and the closing freshness check in
  `validation.md`. [SC-046-3; Principle VI]
