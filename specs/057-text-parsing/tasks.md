# Tasks — 057-text-parsing

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only): no
  change to `specifications/strread` since `1156c97` or to
  `specifications/asManyOfPattern` since `fc9f804`; no drafts; imports
  byte-identical. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md` (claim ids verified).
  [FR-057-01..31]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (recorded in `validation.md`,
  "Analysis"). [Principle VI]
- [x] **T4** Survey reuse: `textread.rs` has no public parser items
  (D-057-2); RunMat pattern objects are `compat::pattern_object` /
  `pattern_regex`; `extract`, `alphanumericsPattern` and pattern `|` are
  absent (D-057-3). [FR-057-13..22]
- [x] **T5** Write the `strread` tests: 15 `observed_*` tests plus
  documented-choice tests. [FR-057-01..09, 25, 26, 29, 30]
- [x] **T6** Write the `asManyOfPattern` tests: 12 `observed_*` tests, the
  non-normative empty-match test, documented-choice and unresolved tests.
  [FR-057-13..22, 27..31]
- [x] **T7** Register the modules (hand-written `io/filetext/mod.rs`,
  `strings/core/mod.rs`) with stub bodies. Run the filter and record the
  failing run. [Principle V]
- [x] **T8** Implement `strread.rs`. [FR-057-01..09, 25, 26, 29, 30]
- [x] **T9** Implement `as_many_of_pattern.rs`. [FR-057-13..22, 27..30]
- [x] **T10** Run the verification commands: feature tests, the
  strings/filetext regressions, clippy and rustfmt; record in
  `validation.md`. [SC-057-1..3]
- [x] **T11** Write the traceability table and the closing freshness check
  in `validation.md`. [SC-057-3; Principle VI]
