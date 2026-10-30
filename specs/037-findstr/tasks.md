# Tasks — 037-findstr (spec 0.3.0)

Order is normative (test-first). Every task cites requirements. Ticks mark
work actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  `8614c5b` = HEAD = origin/main; no diff or working-tree change under
  `specifications/findstr`. [Principle IV]
- [x] **T1** Re-do B1 for 0.3.0: `spec.md` and `provenance.md` (pin
  `8614c5b`, 14 claim ids verified against the imported `provenance.yaml`).
  [FR-037-01..12]
- [x] **T2** Re-do B2: `plan.md`, this file. [all]
- [x] **T3** Rewrite the tests to the 0.3.0 observations: one `observed_*`
  test per observed case (17). [FR-037-01..07; SC-037-1]
- [x] **T4** Write the documented-choice and unobserved-class tests. Remove
  the 0.2.0-era implementation, register the test-only module, run the
  filter, and record the failure (0 passed / 21 failed for findstr, all
  `RunMat:UndefinedFunction`). [FR-037-08..12]
- [x] **T5** Implement `findstr.rs` against 0.3.0: variadic arity errors,
  text/numeric sequences, shorter-in-longer search, `0×0` empty result.
  [FR-037-01..12]
- [x] **T6** Register: `pub mod findstr;` in the generated
  `strings/search/mod.rs` (flagged for the maintainer README). [SC-037-2]
- [x] **T7** Run the verification commands (feature tests,
  `strings::search::`, registry, `compatibility::`, clippy, rustfmt check);
  record them in `validation.md`. [SC-037-1..3]
- [x] **T8** Traceability table (claim → FR → test → result) in
  `validation.md`. [SC-037-3; Principle VI]
