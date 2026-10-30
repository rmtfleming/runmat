# Tasks — 052-data-n2 (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for the three specs
  (fetch; diff `8f72edb` vs `HEAD` and `origin/main`; status). [provenance]
- [x] **T1** Inspect upstream graph subsystem (`GraphData`, `graph_data`,
  `graph_object`, `parse_string_vector`), the categorical object
  (`Codes`/`Categories`, `categorical_categories`) and file I/O conventions
  (`fileread` path handling, `runmat_filesystem`). (no code changes)
- [x] **T2** Write `addnode.rs`, `categories.rs`, `textread.rs` with
  registered stubs (descriptors, bodies returning an `Internal` "not yet
  implemented" error) and the observed normative tests. [FR-052-01..15;
  SC-052-1]
- [x] **T3** Write the `rejects_*` and `unresolved_choice_*` tests.
  [FR-052-04, FR-052-15, FR-052-17]
- [x] **T4** Register the three modules and run the test filter against the
  stubs; record the failing run in `validation.md`.
- [x] **T5** Implement `addnode`. [FR-052-01..04, FR-052-16]
- [x] **T6** Implement `categories`. [FR-052-05..07]
- [x] **T7** Implement `textread`. [FR-052-08..15]
- [x] **T8** Run the verification commands (feature tests, graph/table/io
  suites, rustfmt, clippy); record output in `validation.md`. [SC-052-2,
  SC-052-4]
- [x] **T9** Traceability table (claim → FR → test → result), documented
  choices and deviations in `validation.md`; final freshness check.
  [SC-052-3; Principle VI]
