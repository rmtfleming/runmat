# Tasks — 068-web

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only): no
  change to `specifications/web` since `e364206`; no drafts; imports
  byte-identical. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (claim ids verified).
  [FR-068-01..15]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (recorded in `validation.md`,
  "Analysis"). [Principle VI]
- [x] **T4** Write the `web` tests: 14 `observed_*` tests, documented-choice,
  deviation and unresolved tests, all with a recording launcher.
  [FR-068-01..15]
- [x] **T5** Register the module in the hand-written `io/http/mod.rs` with a
  stub body. Run the filter and record the failing run. [Principle V]
- [x] **T6** Implement `web.rs` (request building, launcher hook, outputs).
  [FR-068-01..14]
- [x] **T7** Run the verification commands: feature tests, the `io::http`
  regressions, clippy and rustfmt. Record the results in `validation.md`.
  [SC-068-1..3]
- [x] **T8** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-068-3; Principle VI]
