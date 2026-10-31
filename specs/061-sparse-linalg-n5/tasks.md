# Tasks — 061-sparse-linalg-n5

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only): no
  change to `specifications/{lsqminnorm,sprandn,sprank}` since `1c91aa1`;
  no drafts; imports byte-identical. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (claim ids verified).
  [FR-061-01..40]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (recorded in `validation.md`,
  "Analysis"). [Principle VI]
- [x] **T4** Write the `lsqminnorm` tests: 17 `observed_*` tests plus
  documented-choice and unresolved tests. [FR-061-01..17]
- [x] **T5** Write the `sprank` tests: 14 `observed_*` tests plus
  documented-choice and unresolved tests. [FR-061-18..28, 40]
- [x] **T6** Write the `sprandn` tests: 11 `observed_*` tests plus
  documented-choice and unresolved tests. [FR-061-29..40]
- [x] **T7** Register the modules in the hand-written `solve/mod.rs`,
  `math/sparse/mod.rs` and `array/creation/sparse/mod.rs` with stub
  bodies. Run the filter and record the failing run. [Principle V]
- [x] **T8** Implement `lsqminnorm.rs`. [FR-061-01..16]
- [x] **T9** Implement `sprank.rs`. [FR-061-18..28]
- [x] **T10** Implement `sprandn.rs`. [FR-061-29..39]
- [x] **T11** Run the verification commands: feature tests, the
  linalg/sparse regressions, clippy and rustfmt. Record the results in
  `validation.md`. [SC-061-1..3]
- [x] **T12** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-061-3; Principle VI]
