# Tasks — 054-types-structs-text

The order is normative (test-first). Every task cites requirements, and a
task is ticked only once the work is actually done.

- [x] **T0** Freshness check of the source spec repository (read-only):
  no change under the four spec directories since their pins (`1156c97`,
  `fc9f804`) and no working-tree draft. [Principle IV]
- [x] **T1** B1: write `spec.md` and `provenance.md`; verify claim ids.
  [FR-054-01..28]
- [x] **T2** B2: write `plan.md` and this file. [all]
- [x] **T3** B3: cross-artifact analysis (claims ↔ FRs ↔ planned tests,
  placement, generated parents). [Principle VI]
- [x] **T4** Write `isfloat_tests`: 17 `observed_*` tests plus
  documented-choice tests. [FR-054-01..04, 23, 24, 28]
- [x] **T5** Write `isnat_tests`: 6 `observed_*` tests plus the
  non-normative error cases as documented-choice tests.
  [FR-054-05..07, 23, 25, 28]
- [x] **T6** Write `struct2cell_tests`: 8 `observed_*` tests plus the
  non-normative error cases as documented-choice tests.
  [FR-054-08..12, 23, 26, 28]
- [x] **T7** Write `strvcat_tests`: 14 `observed_*` tests plus
  documented-choice tests. [FR-054-13..23, 27, 28]
- [x] **T8** Register the modules with stub bodies; run the filter and
  record the failing run. [Principle V]
- [x] **T9** Implement `isfloat.rs`. [FR-054-01..04, 23, 24]
- [x] **T10** Implement `isnat.rs`. [FR-054-05..07, 23, 25]
- [x] **T11** Implement `struct2cell.rs`. [FR-054-08..12, 23, 26]
- [x] **T12** Implement `strvcat.rs`. [FR-054-13..23, 27]
- [x] **T13** Run the verification commands (feature tests, regressions,
  clippy, rustfmt); record results in `validation.md`. [SC-054-1..3]
- [x] **T14** Write the traceability table (claim → FR → test → result) and
  the closing freshness check in `validation.md`. [SC-054-3; Principle VI]
