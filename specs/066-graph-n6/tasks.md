# Tasks — 066-graph-n6 (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for both specs
  (fetch; log `e364206..origin/main`; status; comparison of the imported
  copies). [provenance]
- [x] **T1** Inspect the upstream graph subsystem (`GraphData`,
  `graph_data`, `edges_table`, `nodes_table`), F59's `paths.rs` helpers,
  F62's `reordernodes` table handling and the table `select_rows` helper.
  (no code changes)
- [x] **T2** Write `subgraph.rs` and `findnode.rs` with registered stubs
  (bodies returning `RunMat:<fn>:Internal` "not yet implemented") and the
  observed normative tests. [FR-066-01..17; SC-066-1]
- [x] **T3** Write the `rejects_*`, `unresolved_choice_*` and ignored
  `deviation_*` tests. [FR-066-09..10, FR-066-15..17, FR-066-18]
- [x] **T4** Register `mod findnode;` / `mod subgraph;`, run the test
  filter against the stubs; record the failing run in `validation.md`.
- [x] **T5** Implement `subgraph`. [FR-066-01..10, U-066-1..7]
- [x] **T6** Implement `findnode`. [FR-066-11..17, U-066-1..3, U-066-8..9]
- [x] **T7** Run the verification commands (feature tests, ignored tests,
  graph suite, rustfmt, clippy); record output in `validation.md`.
  [SC-066-2, SC-066-4]
- [x] **T8** Traceability table (claim → FR → test → result), documented
  choices and deviations in `validation.md`; final freshness check.
  [SC-066-3; Principle VI]
