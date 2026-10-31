# Tasks — 059-graph-n3n4 (specs 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for the three specs
  (fetch; log `1156c97..origin/main` / `fc9f804..origin/main`; status;
  byte comparison of the imported copies). [provenance]
- [x] **T1** Inspect the upstream graph subsystem (`GraphData`,
  `graph_data`, `graph_object`, table helpers, parsing helpers) and the
  F36/F43 exemplars. (no code changes)
- [x] **T2** Write `isomorphism.rs` and `paths.rs` with registered stubs
  (descriptors, bodies returning `RunMat:<fn>:Internal` "not yet
  implemented") and the observed normative tests. [FR-059-01..16; SC-059-1]
- [x] **T3** Write the `rejects_*`, `unresolved_choice_*` and ignored
  `deviation_*` tests. [FR-059-06, FR-059-12, FR-059-16, FR-059-17]
- [x] **T4** Register `mod isomorphism;` / `mod paths;` and run the test
  filter against the stubs; record the failing run in `validation.md`.
- [x] **T5** Implement `isomorphism`. [FR-059-01..06, U-059-4, U-059-5]
- [x] **T6** Implement `allpaths`. [FR-059-07..12, U-059-6]
- [x] **T7** Implement `inedges`. [FR-059-13..16, U-059-7]
- [x] **T8** Run the verification commands (feature tests, graph suite,
  rustfmt, clippy); record output in `validation.md`. [SC-059-2, SC-059-4]
- [x] **T9** Traceability table (claim → FR → test → result), documented
  choices and deviations in `validation.md`; final freshness check.
  [SC-059-3; Principle VI]
