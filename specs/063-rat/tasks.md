# Tasks — 063-rat (spec 0.2.0)

Order is normative (test-first). Every task cites requirements. Only work
actually done is ticked.

- [x] **T0** Freshness check of the source repository for `rat` (fetch;
  log `1c91aa1..origin/main`; status; comparison of the imported copy).
  [provenance]
- [x] **T1** Inspect the clean-room numeric exemplar (`sec.rs`, F55) and
  the value/tensor APIs. (no code changes)
- [x] **T2** Write `rat.rs` with a registered stub (body returning
  `RunMat:rat:Internal` "not yet implemented") and the observed tests.
  [FR-063-01..08; SC-063-1]
- [x] **T3** Write the `supported_*` and `unresolved_choice_*` tests.
  [D-063-3, FR-063-09]
- [x] **T4** Register `pub(crate) mod rat;`, run the test filter against
  the stub; record the failing run in `validation.md`.
- [x] **T5** Implement `rat`. [FR-063-01..09]
- [x] **T6** Run the verification commands; record output in
  `validation.md`. [SC-063-2, SC-063-4]
- [x] **T7** Traceability table, documented choices and final freshness
  check in `validation.md`. [SC-063-3; Principle VI]
