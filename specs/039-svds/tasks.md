# Tasks — 039-svds (spec 0.3.0)

Order is normative (test-first). Every task cites requirements. Ticks mark
work actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  `8614c5b` = HEAD = origin/main; no diff or working-tree change under
  `specifications/svds`. [Principle IV]
- [x] **T1** Re-do B1 for 0.3.0: `spec.md` and `provenance.md` (pin
  `8614c5b`, 14 claim ids verified against the imported `provenance.yaml`).
  [FR-039-01..16]
- [x] **T2** Re-do B2: `plan.md`, this file. [all]
- [x] **T3** Rewrite the tests to the 0.3.0 observations: one `observed_*`
  test per observed case (19 cases, 17 tests). [FR-039-01..07; SC-039-1]
- [x] **T4** Write the documented-choice tests and the q-options test.
  Remove the 0.2.0-era implementation, register the test-only module, run
  the filter, and record the failure (0 passed / 26 failed for svds, all
  `RunMat:UndefinedFunction`). [FR-039-08..16]
- [x] **T5** Implement `svds.rs` against 0.3.0: descriptor (6 signatures,
  6 errors), input handling, `k`/sigma parsing, selection, `[U,S,V]` from
  the economy `svd`. [FR-039-01..16]
- [x] **T6** Register: `pub mod svds;` in `factor/mod.rs`. [SC-039-2]
- [x] **T7** Run the verification commands (feature tests, `factor::svd::`,
  registry, `compatibility::`, clippy, rustfmt check); record them in
  `validation.md`. [SC-039-1..3]
- [x] **T8** Traceability table (claim → FR → test → result) in
  `validation.md`. [SC-039-3; Principle VI]
