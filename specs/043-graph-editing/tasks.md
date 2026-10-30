# Tasks — 043-graph-editing (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for the four specs
  (fetch; diff `c65e238` vs `HEAD` and `origin/main`; status). [provenance]
- [x] **T1** Inspect the upstream graph subsystem (`GraphData`, `Edge`,
  `graph_data`, `graph_object`, `edges_table` weight rule, parsing helpers)
  and the F36 exemplar. [FR-043-19] (no code changes)
- [x] **T2** Write `editing.rs` with registered stubs (descriptors, bodies
  returning an internal "not implemented" error) and the observed normative
  tests for all four builtins. [FR-043-01..18; SC-043-1]
- [x] **T3** Write the `rejects_*`, `unresolved_choice_*` and ignored
  `deviation_*` tests. [FR-043-06, FR-043-12, FR-043-16, FR-043-18,
  FR-043-20]
- [x] **T4** Register `mod editing;` and run the test filter against the
  stubs; record the failing run in `validation.md`.
- [x] **T5** Implement `rmnode`. [FR-043-01..06]
- [x] **T6** Implement `addedge`. [FR-043-07..12]
- [x] **T7** Implement `rmedge`. [FR-043-13..16]
- [x] **T8** Implement `laplacian`. [FR-043-17, FR-043-18]
- [x] **T9** Run the verification commands (feature tests, graph suite,
  rustfmt, clippy); record output in `validation.md`. [SC-043-2, SC-043-4]
- [x] **T10** Traceability table (claim → FR → test → result), documented
  choices and deviations in `validation.md`; final freshness check.
  [SC-043-3; Principle VI]
