# Tasks — 067-numeric-n6

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only): no
  change to `specifications/{normest,lsqnonneg}` since `e364206`; no drafts;
  imports byte-identical. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (claim ids verified).
  [FR-067-01..41]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (recorded in `validation.md`,
  "Analysis"). [Principle VI]
- [x] **T4** Write the `normest` tests: one `observed_*` test per observed
  case plus documented-choice and unresolved tests. [FR-067-01..16]
- [x] **T5** Write the `lsqnonneg` tests: one `observed_*` test per observed
  case plus documented-choice and unresolved tests. [FR-067-20..41]
- [x] **T6** Register the modules in the hand-written `solve/mod.rs` and
  `optim/mod.rs` with stub bodies. Run the filter and record the failing
  run. [Principle V]
- [x] **T7** Implement `normest.rs`. [FR-067-01..15]
- [x] **T8** Implement `lsqnonneg.rs`. [FR-067-20..40]
- [x] **T9** Run the verification commands: feature tests, the linalg/optim
  regressions, clippy and rustfmt. Record the results in `validation.md`.
  [SC-067-1..3]
- [x] **T10** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-067-3; Principle VI]
