# Tasks — 055-numeric-n3n4

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only): no
  change to `specifications/{sec,orth}` since `1156c97` or to
  `specifications/condest` since `fc9f804`; no drafts; imports
  byte-identical. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (claim ids verified).
  [FR-055-01..33]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (recorded in `validation.md`,
  "Analysis"). [Principle VI]
- [x] **T4** Write the `sec` tests: 15 `observed_*` tests plus
  documented-choice and unresolved tests. [FR-055-01..08, 29..33]
- [x] **T5** Write the `orth` tests: 15 `observed_*` tests plus
  documented-choice and unresolved tests. [FR-055-09..17, 27..31, 33]
- [x] **T6** Write the `condest` tests: 12 `observed_*` tests plus
  documented-choice and unresolved tests. [FR-055-18..26, 29..31, 33]
- [x] **T7** Register the modules (hand-written `factor/mod.rs`,
  `solve/mod.rs`; one hand-added line in generated `trigonometry/mod.rs`)
  with stub bodies. Run the filter and record the failing run.
  [Principle V]
- [x] **T8** Implement `sec.rs`. [FR-055-01..08, 29..32]
- [x] **T9** Implement `orth.rs`. [FR-055-09..16, 27..31]
- [x] **T10** Implement `condest.rs`. [FR-055-18..26, 29..31]
- [x] **T11** Run the verification commands: feature tests, the
  trigonometry/linalg regressions, clippy and rustfmt. Record the results
  in `validation.md`. [SC-055-1..3]
- [x] **T12** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-055-3; Principle VI]
