# Tasks — 062-graph-n5 (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for both specs
  (fetch; log `1c91aa1..origin/main`; status; comparison of the imported
  copies). [provenance]
- [x] **T1** Inspect the upstream graph subsystem (`GraphData`,
  `graph_data`, `edges_table`, `nodes_table`), F59's `inedges` helpers and
  the table `select_rows` helper. (no code changes)
- [x] **T2** Write `outedges.rs` and `reordernodes.rs` with registered
  stubs (bodies returning `RunMat:<fn>:Internal` "not yet implemented") and
  the observed normative tests. [FR-062-01..13; SC-062-1]
- [x] **T3** Write the `rejects_*`, `unresolved_choice_*` and ignored
  `deviation_*` tests. [FR-062-07, FR-062-13, FR-062-14]
- [x] **T4** Register `mod outedges;` / `mod reordernodes;`, widen the F59
  helpers to `pub(super)`, run the test filter against the stubs; record
  the failing run in `validation.md`.
- [x] **T5** Implement `outedges`. [FR-062-01..07, U-062-1..5]
- [x] **T6** Implement `reordernodes`. [FR-062-08..13, U-062-3, U-062-6..9]
- [x] **T7** Run the verification commands (feature tests, graph suite,
  rustfmt, clippy); record output in `validation.md`. [SC-062-2, SC-062-4]
- [x] **T8** Traceability table (claim → FR → test → result), documented
  choices and deviations in `validation.md`; final freshness check.
  [SC-062-3; Principle VI]
